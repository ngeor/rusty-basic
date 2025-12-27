use rusty_common::{AtPos, Positioned};

/// Visits an element by ref and returns a new value.
pub trait RefToValueVisitor<I, O, E> {
    fn ref_to_value_visit(&mut self, element: &I) -> Result<O, E>;
}

// Blanket implementation for Vec

impl<P, I, O, E> RefToValueVisitor<Vec<I>, Vec<O>, E> for P
where
    P: RefToValueVisitor<I, O, E>,
{
    fn ref_to_value_visit(&mut self, element: &Vec<I>) -> Result<Vec<O>, E> {
        element.iter().map(|p| self.ref_to_value_visit(p)).collect()
    }
}

// Blanket implementation for Positioned

impl<P, I, O, E> RefToValueVisitor<Positioned<I>, Positioned<O>, Positioned<E>> for P
where
    P: RefToValueVisitor<I, O, E>,
{
    fn ref_to_value_visit(
        &mut self,
        element: &Positioned<I>,
    ) -> Result<Positioned<O>, Positioned<E>> {
        let Positioned { element, pos } = element;
        match self.ref_to_value_visit(element) {
            Ok(ok) => Ok(ok.at_pos(*pos)),
            Err(err) => Err(err.at_pos(*pos)),
        }
    }
}
