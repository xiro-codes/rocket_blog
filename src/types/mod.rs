use rocket::{
    outcome::Outcome,
    request::{self, FromRequest},
    http::{
        Status,
    },
    response::Responder,
    Request, Response,
    fs::FileServer,
};
use std::{
    fs::File,
    io::{self, prelude::*, SeekFrom, Cursor},
    sync::Mutex,
    path::{Path, PathBuf},
};pub struct StreamedFile {
    file: Mutex<File>,
    size: u64,
    range: Option<std::ops::Range<u64>>,
    content_type: String,
}
impl StreamedFile {
    pub fn new(path: &str, range: Option<std::ops::Range<u64>>) -> io::Result<Self> {
        let file = File::open(path)?;
        let size = file.metadata()?.len();
        let content_type = mime_guess::from_path(path)
            .first_or_octet_stream()
            .to_string();
        Ok(Self {
            file: Mutex::new(file),
            content_type,
            size,
            range
        })
    }
}
impl<'r> Responder<'r, 'static> for StreamedFile {
    fn respond_to(self, req: &'r Request<'_>) -> rocket::response::Result<'static> {
        let mut response = Response::build();
        response.raw_header("Accept-Ranges", "bytes");
        response.raw_header("Content-Type", self.content_type);
        response.raw_header("Content-Length", self.size.to_string());

        // Handle partial content
        if let Some(mut range) = self.range {
            let mut file = self.file.lock().unwrap();

            // Correct open-ended ranges based on the file size.
            if range.start >= self.size {
                return Err(Status::RangeNotSatisfiable);
            }
            if range.end > self.size {
                range.end = self.size;
            }
            let length = range.end - range.start;

            // Seek to the starting position and read the bytes.
            file.seek(SeekFrom::Start(range.start)).unwrap();
            let mut buf = vec![0u8; length as usize];
            let bytes_read = file.read(&mut buf).unwrap();

            // Build the response with the partial content.
            response.status(Status::PartialContent)
                .raw_header("Content-Range", format!(
                    "bytes {}-{}/{}", 
                    range.start, 
                    range.start + bytes_read as u64, 
                    self.size))
                .raw_header("Content-Length", (bytes_read as u64).to_string())
                .sized_body(bytes_read, io::Cursor::new(buf));

        } else {
            // Handle full content (no range header)
            let mut file = self.file.lock().unwrap();
            let mut buf = Vec::new();
            file.read_to_end(&mut buf).unwrap();
            response.status(Status::Ok)
                .sized_body(self.size as usize, Cursor::new(buf));
        }

        response.ok()    }
}
pub struct HttpRange(pub std::ops::Range<u64>);

#[rocket::async_trait]
impl<'r> FromRequest<'r> for HttpRange {
    type Error = ();
    async fn from_request(req: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
        let file_size: u64 = 1_000; 

        if let Some(range_header) = req.headers().get_one("Range") {
            let range_str = range_header.trim_start_matches("bytes=");
            let parts: Vec<&str> = range_str.split('-').collect();

            let start = parts[0].parse::<u64>().ok();
            let end = parts.get(1).and_then(|s| s.parse::<u64>().ok());

            match (start, end) {
                // Case 1: `Range: bytes=start-end`
                (Some(start), Some(end)) => {
                    // Check for invalid ranges
                    if start >= end || end >= file_size {
                        Outcome::Error((Status::RangeNotSatisfiable, ()))
                    } else {
                        // The `Range` struct is exclusive of the end value, so we add 1
                        Outcome::Success(HttpRange(start..end + 1))
                    }
                }
                // Case 2: `Range: bytes=start-`
                (Some(start), None) => {
                    if start >= file_size {
                        Outcome::Error((Status::RangeNotSatisfiable, ()))
                    } else {
                        // Requesting from start to the end of the file
                        Outcome::Success(HttpRange(start..file_size))
                    }
                }
                // Case 3: `Range: bytes=-end`
                (None, Some(end)) => {
                    let start = file_size.saturating_sub(end);
                    if start >= file_size {
                        Outcome::Error((Status::RangeNotSatisfiable, ()))
                    } else {
                        // Requesting the last `end` bytes of the file
                        Outcome::Success(HttpRange(start..file_size))
                    }
                }
                // Case 4: Invalid format
                _ => {
                    Outcome::Error((Status::RangeNotSatisfiable, ()))
                }
            }
        } else {
            // No Range header, so this is a request for the entire file.
            // We return `Outcome::Forward` to let a different handler deal with it.
            Outcome::Forward(Status::Ok)
        }
    }
}
