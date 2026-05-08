# const-generics

**Prusti limitation:** Const generics cause a panic inside Prusti.

Functions or types parameterised by a `const` generic parameter (e.g.
`fn foo<const N: usize>(...)`) cause Prusti's internal
`get_body_with_borrowck_facts` call to panic, preventing any analysis of the
function body.

**Sources:**
- <https://github.com/viperproject/prusti-dev/issues/1195>
