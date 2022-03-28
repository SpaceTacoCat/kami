use crate::BoundingBox;

pub type Percentage = f32;

pub struct Layout {
    pub root: Split,
}

pub enum Split {
    Singleton(Frame),
    Vertical(Vec<(Split, Percentage)>),
    Horizontal(Vec<(Split, Percentage)>),
}

pub struct Frame {
    pub buffer_index: usize,
}

impl Layout {
    pub fn new() -> Self {
        Self {
            root: Split::Singleton(Frame { buffer_index: 0 }),
        }
    }
}

impl Split {
    fn build_bounding_boxes(&self, container: BoundingBox, accumulator: &mut Vec<BoundingBox>) {
        match self {
            Split::Singleton(_) => {
                accumulator.push(container);
            }
            Split::Vertical(sub) => {
                let mut accumulated_top = container.top;

                for (item, percentage) in sub {
                    let height = container.height * percentage;
                    accumulated_top += height;

                    item.build_bounding_boxes(
                        BoundingBox {
                            left: container.left,
                            top: accumulated_top,
                            width: container.width,
                            height,
                        },
                        accumulator,
                    );
                }
            }
            Split::Horizontal(sub) => {
                let mut accumulated_left = container.left;

                for (item, percentage) in sub {
                    let width = container.width * percentage;
                    accumulated_left += width;

                    item.build_bounding_boxes(
                        BoundingBox {
                            left: accumulated_left,
                            top: container.top,
                            width,
                            height: container.height,
                        },
                        accumulator,
                    );
                }
            }
        }
    }
}

impl Default for Layout {
    fn default() -> Self {
        Self::new()
    }
}
