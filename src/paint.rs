use crate::cli::Color;
use crate::error::Error;
use ab_glyph::FontArc;
use ab_glyph::PxScaleFont;
use ab_glyph::ScaleFont;

pub trait Paintable {
    fn width(&self) -> usize;
    fn height(&self) -> usize;
    fn set_pixel(&mut self, x: usize, y: usize, color: Color) -> Result<(), Error>;
    fn get_pixel(&self, x: usize, y: usize) -> Result<Color, Error>;
    fn draw_pixel(&mut self, x: usize, y: usize, color: Color) -> Result<(), Error> {
        self.set_pixel(x, y, color.blend(&self.get_pixel(x, y)?))
    }
    fn slice<'slice>(
        &'slice mut self,
        x: usize,
        y: usize,
        width: usize,
        height: usize,
    ) -> Result<PaintableSlice<'slice, Self>, Error>
    where
        Self: Sized,
    {
        if self.height() < y + height || self.width() < x + width {
            Err(Error::PointOutbound)
        } else {
            Ok(PaintableSlice::new(self, x, y, height, width))
        }
    }
}

pub trait Paint {
    fn paint(&self, canvas: &mut impl Paintable) -> Result<(), Error>;
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
            .get_mut(y * self.width * 4 + x * 4..y * self.width * 4 + x * 4 + 4)
            .map(|v| v.try_into().ok())?
    }

    fn get_buffer(&self, x: usize, y: usize) -> Option<&[u8; 4]> {
        self.buffer
            .get(y * self.width * 4 + x * 4..y * self.width * 4 + x * 4 + 4)
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
    fn set_pixel(&mut self, x: usize, y: usize, color: Color) -> Result<(), Error> {
        if y >= self.height || x >= self.width {
            return Err(Error::PointOutbound);
        }
        self.parent_canvas.set_pixel(x + self.x, y + self.y, color)
    }

    fn get_pixel(&self, x: usize, y: usize) -> Result<Color, Error> {
        if y >= self.height || x >= self.width {
            return Err(Error::PointOutbound);
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
    fn set_pixel(&mut self, x: usize, y: usize, color: Color) -> Result<(), Error> {
        if self.height < y || self.width < x {
            return Err(Error::PointOutbound);
        }
        *self.get_buffer_mut(x, y).ok_or(Error::PointOutbound)? = (&color).into();
        Ok(())
    }

    fn get_pixel(&self, x: usize, y: usize) -> Result<Color, Error> {
        Ok(self.get_buffer(x, y).ok_or(Error::PointOutbound)?.into())
    }

    fn width(&self) -> usize {
        self.width
    }

    fn height(&self) -> usize {
        self.height
    }
}

#[derive(Debug)]
pub struct Text {
    //content: String,
    content: String,
    fonts: Vec<PxScaleFont<FontArc>>,
    fg_color: Color,
    bg_color: Color,
}

impl<'font> Text {
    /// Get right font for a character, seeking in all fonts registred in the `fonts` vec.
    ///
    /// The last font was returned if there're no suitable font.
    fn get_font(&self, ch: char) -> &PxScaleFont<FontArc> {
        for i in &self.fonts {
            let glyph_id = i.glyph_id(ch);
            if glyph_id.0 == 0 {
                continue;
            }
            return i;
        }
        return self.fonts.last().unwrap(); // Notdef
    }

    pub fn new(
        content: String,
        fonts: Vec<PxScaleFont<FontArc>>,
        fg_color: Color,
        bg_color: Color,
    ) -> Self {
        Self {
            content,
            fonts,
            fg_color,
            bg_color,
        }
    }

    pub fn get_region(&self) -> (f32, f32) {
        let mut cursor = ab_glyph::point(0., self.fonts.first().unwrap().ascent());
        for i in self.content.chars() {
            let font = self.get_font(i);
            let glyph_id = font.glyph_id(i);
            cursor.x += font.h_advance(glyph_id);
        }
        (cursor.x, self.fonts.first().unwrap().height())
    }
}

impl Paint for Text {
    fn paint(&self, canvas: &mut impl Paintable) -> Result<(), Error> {
        let mut cursor = ab_glyph::point(0., self.fonts.first().unwrap().ascent());
        for i in self.content.chars() {
            let font = self.get_font(i);
            let scale: ab_glyph::PxScale = font.scale();
            let glyph_id = font.glyph_id(i);
            let glyph = glyph_id.with_scale_and_position(scale, cursor);
            let outline = font.outline_glyph(glyph).unwrap_or_else(
                || {
                    font.outline_glyph(ab_glyph::GlyphId(0).with_scale_and_position(scale, cursor))
                        .unwrap()
                }, // There MUST be at least 1 glyphs
            );
            if i != ' ' {
                outline.draw(|x, y, v| {
                    canvas
                        .draw_pixel(
                            (x as f32 + outline.px_bounds().min.x) as usize,
                            (y as f32 + outline.px_bounds().min.y) as usize,
                            self.fg_color
                                .with_alpha((v * 256.) as u8)
                                .blend(&self.bg_color),
                        )
                        .unwrap()
                });
            }
            cursor.x += font.h_advance(glyph_id);
        }
        Ok(())
    }
}
