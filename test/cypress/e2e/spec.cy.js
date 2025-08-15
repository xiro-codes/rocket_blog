describe('template spec', () => {
	it('passes', () => {
		cy.visit("http://localhost:8000/blog")
		cy.get("input[name='username']").type("admin")
	})
})
