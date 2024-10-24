use crate::cli::Color;

pub trait Paintable {
    fn width(&self) -> usize;
    fn height(&self) -> usize;
    fn set_pixel(&mut self, x: usize, y: usize, color: Color) -> Result<(), ()>;
    fn get_pixel(&self, x: usize, y: usize) -> Result<Color, ()>;
    fn draw_pixel(&mut self, x: usize, y: usize, color: Color) -> Result<(), ()> {
        self.set_pixel(x, y, color.blend(&self.get_pixel(x, y)?))
    }
    fn slice<'slice>(
        &'slice mut self,
        x: usize,
        y: usize,
        width: usize,
        height: usize,
    ) -> Result<PaintableSlice<'slice, Self>, ()>
    where
        Self: Sized,
    {
        if self.height() < y + height || self.width() < x + width {
            Err(())
        } else {
            Ok(PaintableSlice::new(self, x, y, height, width))
        }
    }
}

pub trait Paint {
    fn paint(&self, canvas: &mut impl Paintable) -> Result<(), ()>;
}

pub struct Canvas<'buffer> {
    height: usize,
    width: usize,
    buffer: &'buffer mut [u8],
}

impl<'buffer> Canvas<'buffer> {
    pub fn new(height: usize, width: usize, buffer: &'buffer mut [u8]) -> Self {
        Self {
            height,
            width,
            buffer,
        }
    }

    fn get_buffer_mut(&mut self, x: usize, y: usize) -> Option<&mut [u8; 4]> {
        self.buffer
            .get_mut(y * self.width * 4 + x*4 .. y * self.width * 4 + x*4 + 4)
            .map(|v| v.try_into().ok())?
    }

    fn get_buffer(&self, x: usize, y: usize) -> Option<&[u8; 4]> {
        self.buffer
            .get(y * self.width * 4 + x*4 .. y * self.width * 4 + x*4 + 4)
            .map(|v| v.try_into().ok())?
    }
}

pub struct PaintableSlice<'parent, P>
where
    P: Paintable,
{
    parent_canvas: &'parent mut P,
    x: usize,
    y: usize,
    height: usize,
    width: usize,
}

impl<'parent, P> PaintableSlice<'parent, P>
where
    P: Paintable,
{
    pub fn new(
        parent_canvas: &'parent mut P,
        x: usize,
        y: usize,
        height: usize,
        width: usize,
    ) -> Self {
        Self {
            parent_canvas,
            x,
            y,
            height,
            width,
        }
    }
}

impl<P> Paintable for PaintableSlice<'_, P>
where
    P: Paintable,
{
    fn set_pixel(&mut self, x: usize, y: usize, color: Color) -> Result<(), ()> {
        if y >= self.height || x >= self.width {
            return Err(());
        }
        self.parent_canvas.set_pixel(x + self.x, y + self.y, color)
    }

    fn get_pixel(&self, x: usize, y: usize) -> Result<Color, ()> {
        if y >= self.height || x >= self.width {
            return Err(());
        }
        self.parent_canvas.get_pixel(x + self.x, y + self.y)
    }

    fn width(&self) -> usize {
        self.width
    }

    fn height(&self) -> usize {
        self.height
    }
}

impl Paintable for Canvas<'_> {
    fn set_pixel(&mut self, x: usize, y: usize, color: Color) -> Result<(), ()> {
        if self.height < y || self.width < x {
            return Err(());
        }
        *self.get_buffer_mut(x, y).ok_or(())? = (&color).into();
        Ok(())
    }

    fn get_pixel(&self, x: usize, y: usize) -> Result<Color, ()> {
        Ok(self.get_buffer(x, y).ok_or(())?.into())
    }

    fn width(&self) -> usize {
        self.width
    }

    fn height(&self) -> usize {
        self.height
    }
}

#[derive(Debug)]
pub struct Text<'font> {
    content: String,
    font: rusttype::Font<'font>,
    fg_color: Color,
    bg_color: Color,
}

impl<'font> Text<'font> {
    pub fn new(
        content: String,
        font: rusttype::Font<'font>,
        fg_color: Color,
        bg_color: Color,
    ) -> Self {
        Self {
            content,
            font,
            fg_color,
            bg_color,
        }
    }
}

impl Paint for Text<'_> {
    fn paint(&self, canvas: &mut impl Paintable) -> Result<(), ()> {
        let scale = rusttype::Scale::uniform(20.);
        let v_metrics = self.font.v_metrics(scale);
        let start = rusttype::point(5.0, v_metrics.ascent);
        let glyphs = self.font.layout(&self.content, scale, start);
        for glyph in glyphs {
            if let Some(bounding_box) = glyph.pixel_bounding_box() {
                glyph.draw(|x, y, v| {
                    let blend = self
                            .fg_color
                            .with_alpha((v * 255.) as u8)
                            .blend(&self.bg_color);
                    canvas.draw_pixel(
                        (x + bounding_box.min.x as u32) as usize,
                        (y + bounding_box.min.y as u32) as usize,
                        blend,
                    ).unwrap();
                })
            }
        }
        Ok(())
    }
}
