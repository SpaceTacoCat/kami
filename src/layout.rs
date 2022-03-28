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
    // TODO: Make it init Singleton
    pub fn new() -> Self {
        Self {
            root: Split::Horizontal(vec![
                (Split::Singleton(Frame { buffer_index: 0 }), 0.3),
                (Split::Singleton(Frame { buffer_index: 1 }), 0.7),
            ]),
        }
    }

    pub fn build_bounding_boxes(&self, container: BoundingBox) -> Vec<(BoundingBox, usize)> {
        let mut accumulator = Vec::new();

        self.root.build_bounding_boxes(container, &mut accumulator);

        accumulator
    }
}

impl Split {
    fn build_bounding_boxes(
        &self,
        container: BoundingBox,
        accumulator: &mut Vec<(BoundingBox, usize)>,
    ) {
        match self {
            Split::Singleton(frame) => {
                accumulator.push((container, frame.buffer_index));
            }
            Split::Vertical(sub) => {
                let mut accumulated_top = container.top;

                for (item, percentage) in sub {
                    let height = container.height * percentage;

                    item.build_bounding_boxes(
                        BoundingBox {
                            left: container.left,
                            top: accumulated_top,
                            width: container.width,
                            height,
                        },
                        accumulator,
                    );

                    accumulated_top += height;
                }
            }
            Split::Horizontal(sub) => {
                let mut accumulated_left = container.left;

                for (item, percentage) in sub {
                    let width = container.width * percentage;

                    item.build_bounding_boxes(
                        BoundingBox {
                            left: accumulated_left,
                            top: container.top,
                            width,
                            height: container.height,
                        },
                        accumulator,
                    );

                    accumulated_left += width;
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
