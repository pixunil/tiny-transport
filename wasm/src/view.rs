use na::{Matrix4, Point2, Similarity2, Translation2, Vector2, Vector3};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct View {
    viewport: Vector2<f32>,
    view: Similarity2<f32>,
}

#[wasm_bindgen]
impl View {
    #[wasm_bindgen(constructor)]
    pub fn new(scaling: f32, width: f32, height: f32) -> View {
        View {
            viewport: Vector2::new(width, height),
            view: Similarity2::from_scaling(scaling),
        }
    }

    pub fn resize(&mut self, width: f32, height: f32) {
        self.viewport = Vector2::new(width, height);
    }

    pub fn scaling(&self) -> f32 {
        self.view.scaling()
    }

    #[wasm_bindgen(js_name = calculateViewProjection)]
    pub fn view_projection(&self) -> Vec<f32> {
        let scaling = Vector3::new(2.0 / self.viewport.x, -2.0 / self.viewport.y, 0.0);
        let projection = Matrix4::new_nonuniform_scaling(&scaling);
        let mut view = self
            .view
            .to_homogeneous()
            .insert_row(2, 0.0)
            .insert_column(2, 0.0);
        view[(2, 2)] = 1.0;
        (projection * view).data.to_vec()
    }

    pub fn scroll(&mut self, x: f32, y: f32) {
        let shift = Vector2::new(x, y) / self.scaling();
        self.view *= Translation2::from(shift);
    }

    pub(crate) fn unproject(&self, point: Point2<f32>) -> Point2<f32> {
        self.view.inverse() * (point - self.viewport / 2.0)
    }

    pub fn zoom(&mut self, scaling: f32, x: f32, y: f32) {
        let shift = self.unproject(Point2::new(x, y)).coords;
        self.view *= Translation2::from(shift * (1.0 - scaling));
        self.view.prepend_scaling_mut(scaling);
    }
}

#[cfg(test)]
mod tests {
    use approx::assert_relative_eq;
    use na::{Point2, Transform2};
    use std::f32::EPSILON;

    use super::*;

    fn transform_from_view(view: &View) -> Transform2<f32> {
        let matrix = Matrix4::from_vec(view.view_projection())
            .remove_row(2)
            .remove_column(2);
        Transform2::from_matrix_unchecked(matrix)
    }

    #[test]
    fn test_viewport_scaling() {
        let view = View::new(1.0, 300.0, 200.0);
        let transform = transform_from_view(&view);
        assert_relative_eq!(transform * Point2::new(0.0, 0.0), Point2::new(0.0, 0.0));
        assert_relative_eq!(transform * Point2::new(150.0, 0.0), Point2::new(1.0, 0.0));
        assert_relative_eq!(transform * Point2::new(0.0, 100.0), Point2::new(0.0, -1.0));
        assert_relative_eq!(transform * Point2::new(-150.0, 0.0), Point2::new(-1.0, 0.0));
        assert_relative_eq!(transform * Point2::new(0.0, -100.0), Point2::new(0.0, 1.0));
    }

    #[test]
    fn test_initial_scaling() {
        let view = View::new(0.5, 300.0, 200.0);
        let transform = transform_from_view(&view);
        assert_relative_eq!(transform * Point2::new(0.0, 0.0), Point2::new(0.0, 0.0));
        assert_relative_eq!(transform * Point2::new(300.0, 0.0), Point2::new(1.0, 0.0));
        assert_relative_eq!(transform * Point2::new(0.0, 200.0), Point2::new(0.0, -1.0));
        assert_relative_eq!(transform * Point2::new(-300.0, 0.0), Point2::new(-1.0, 0.0));
        assert_relative_eq!(transform * Point2::new(0.0, -200.0), Point2::new(0.0, 1.0));
    }

    #[test]
    fn test_resize() {
        let mut view = View::new(1.0, 300.0, 200.0);
        view.resize(400.0, 160.0);
        let transform = transform_from_view(&view);
        assert_relative_eq!(transform * Point2::new(0.0, 0.0), Point2::new(0.0, 0.0));
        assert_relative_eq!(transform * Point2::new(200.0, 0.0), Point2::new(1.0, 0.0));
        assert_relative_eq!(transform * Point2::new(0.0, 80.0), Point2::new(0.0, -1.0));
        assert_relative_eq!(transform * Point2::new(-200.0, 0.0), Point2::new(-1.0, 0.0));
        assert_relative_eq!(transform * Point2::new(0.0, -80.0), Point2::new(0.0, 1.0));
    }

    #[test]
    fn test_scroll() {
        let mut view = View::new(1.0, 300.0, 200.0);
        view.scroll(-300.0, 200.0);
        let transform = transform_from_view(&view);
        assert_relative_eq!(
            transform * Point2::new(300.0, -200.0),
            Point2::new(0.0, 0.0)
        );
        assert_relative_eq!(
            transform * Point2::new(450.0, -200.0),
            Point2::new(1.0, 0.0)
        );
        assert_relative_eq!(
            transform * Point2::new(300.0, -100.0),
            Point2::new(0.0, -1.0)
        );
        assert_relative_eq!(
            transform * Point2::new(150.0, -200.0),
            Point2::new(-1.0, 0.0)
        );
        assert_relative_eq!(
            transform * Point2::new(300.0, -300.0),
            Point2::new(0.0, 1.0)
        );
    }

    #[test]
    fn test_zoom() {
        let mut view = View::new(1.0, 300.0, 200.0);
        view.zoom(2.0, 210.0, 120.0);
        let transform = transform_from_view(&view);
        assert_relative_eq!(view.scaling(), 2.0);
        assert_relative_eq!(transform * Point2::new(60.0, 20.0), Point2::new(0.4, -0.2));
        assert_relative_eq!(transform * Point2::new(30.0, 10.0), Point2::new(0.0, 0.0));
        assert_relative_eq!(transform * Point2::new(105.0, 10.0), Point2::new(1.0, 0.0));
        assert_relative_eq!(transform * Point2::new(30.0, 60.0), Point2::new(0.0, -1.0));
        assert_relative_eq!(transform * Point2::new(-45.0, 10.0), Point2::new(-1.0, 0.0));
        assert_relative_eq!(transform * Point2::new(30.0, -40.0), Point2::new(0.0, 1.0));
    }

    #[test]
    fn test_scroll_after_zoom() {
        let mut view = View::new(1.0, 300.0, 200.0);
        view.zoom(2.0, 210.0, 120.0);
        view.scroll(-300.0, 200.0);
        let transform = transform_from_view(&view);
        assert_relative_eq!(view.scaling(), 2.0);
        assert_relative_eq!(
            transform * Point2::new(210.0, -80.0),
            Point2::new(0.4, -0.2),
            epsilon = 2.0 * EPSILON
        );
        assert_relative_eq!(transform * Point2::new(180.0, -90.0), Point2::new(0.0, 0.0));
        assert_relative_eq!(transform * Point2::new(255.0, -90.0), Point2::new(1.0, 0.0));
        assert_relative_eq!(
            transform * Point2::new(180.0, -40.0),
            Point2::new(0.0, -1.0)
        );
        assert_relative_eq!(
            transform * Point2::new(105.0, -90.0),
            Point2::new(-1.0, 0.0)
        );
        assert_relative_eq!(
            transform * Point2::new(180.0, -140.0),
            Point2::new(0.0, 1.0)
        );
    }

    #[test]
    fn test_zoom_after_zoom() {
        let mut view = View::new(1.0, 300.0, 200.0);
        view.zoom(2.0, 210.0, 120.0);
        view.zoom(3.0, 150.0, 100.0);
        let transform = transform_from_view(&view);
        assert_relative_eq!(view.scaling(), 6.0);
        assert_relative_eq!(
            transform * Point2::new(60.0, 20.0),
            Point2::new(1.2, -0.6),
            epsilon = 2.0 * EPSILON
        );
        assert_relative_eq!(transform * Point2::new(30.0, 10.0), Point2::new(0.0, 0.0));
        assert_relative_eq!(transform * Point2::new(55.0, 10.0), Point2::new(1.0, 0.0));
        assert_relative_eq!(
            transform * Point2::new(30.0, 80.0 / 3.0),
            Point2::new(0.0, -1.0)
        );
        assert_relative_eq!(transform * Point2::new(5.0, 10.0), Point2::new(-1.0, 0.0));
        assert_relative_eq!(
            transform * Point2::new(30.0, -20.0 / 3.0),
            Point2::new(0.0, 1.0)
        );
    }

    #[test]
    fn test_unproject() {
        let view = View::new(1.0, 300.0, 200.0);
        assert_relative_eq!(
            view.unproject(Point2::new(150.0, 100.0)),
            Point2::new(0.0, 0.0)
        );
        assert_relative_eq!(
            view.unproject(Point2::new(210.0, 120.0)),
            Point2::new(60.0, 20.0)
        );
    }

    #[test]
    fn test_unproject_after_zoom() {
        let mut view = View::new(1.0, 300.0, 200.0);
        view.zoom(2.0, 210.0, 120.0);
        assert_relative_eq!(
            view.unproject(Point2::new(150.0, 100.0)),
            Point2::new(30.0, 10.0)
        );
        assert_relative_eq!(
            view.unproject(Point2::new(210.0, 120.0)),
            Point2::new(60.0, 20.0)
        );
        assert_relative_eq!(
            view.unproject(Point2::new(90.0, 80.0)),
            Point2::new(0.0, 0.0)
        );
    }
}
