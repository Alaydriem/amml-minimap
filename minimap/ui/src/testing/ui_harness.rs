use amml_ui::font::FontSource;
use amml_ui::text::UiLocale;
use amml_ui::theme::Theme;
use egui::epaint::{ClippedShape, Primitive};
use egui::{
    Context, Event, Key, Modifiers, MouseWheelSource, MouseWheelUnit, OutputCommand, PointerButton, Pos2, RawInput,
    Rect, Shape, TouchPhase, ViewportId, ViewportInfo, pos2, vec2,
};

use crate::runtime::MinimapLocale;
use crate::testing::{HarnessEvent, HarnessInput, PassOutput, ShownText};

// stands in for the manager in tests: one egui context with the amml-ui theme, RawInput built from the
// polled pointer and the queued events the way the manager routes them, and the pass's output collected
pub struct UiHarness {
    ctx: Context,
    prev_left: bool,
    prev_right: bool,
    prev_cursor: Option<Pos2>,
    modifiers: Modifiers,
}

impl UiHarness {
    // the manager sends one line per wheel notch
    const WHEEL_NOTCH: f32 = 120.0;

    pub fn new() -> Self {
        Self {
            ctx: Self::themed(),
            prev_left: false,
            prev_right: false,
            prev_cursor: None,
            modifiers: Modifiers::default(),
        }
    }

    pub fn context(&self) -> &Context {
        &self.ctx
    }

    // the panels' words in the language of `code`, as the manager sets them: empty for English, `en_XA` for the
    // pseudo-locale
    pub fn set_locale(&self, code: &str) {
        UiLocale::set(&self.ctx, MinimapLocale::localizer(code));
    }

    // a fresh context, as the manager makes after a lost device or a caught panic
    pub fn replace_context(&mut self) {
        self.ctx = Self::themed();
        self.prev_cursor = None;
    }

    pub fn run(&mut self, input: &HarnessInput<'_>, out: &mut PassOutput, paint: impl FnMut(&Context)) {
        out.clear();
        let ppp = input.pixels_per_point.max(f32::EPSILON);
        let raw = self.raw(input, ppp);
        let mut output = self.ctx.run_pass(raw, paint);
        out.wants_pointer = self.ctx.egui_wants_pointer_input();
        out.wants_keyboard = self.ctx.egui_wants_keyboard_input();
        for command in &output.platform_output.commands {
            if let OutputCommand::CopyText(text) = command {
                out.clipboard.clone_from(text);
            }
        }
        for (id, deltas) in &output.textures_delta.set {
            for delta in deltas {
                out.textures_set.push((*id, delta.image.size(), delta.options));
            }
        }
        out.textures_freed.extend(output.textures_delta.free.iter().copied());
        output.textures_delta.clear();
        Self::collect(&output.shapes, ppp, out);
        let primitives = self.ctx.tessellate(std::mem::take(&mut output.shapes), output.pixels_per_point);
        for primitive in primitives {
            if let Primitive::Mesh(mesh) = primitive.primitive {
                out.meshes.push(mesh);
            }
        }
    }

    fn themed() -> Context {
        let ctx = Context::default();
        Theme::install(&ctx, FontSource::BuiltIn);
        ctx
    }

    fn raw(&mut self, input: &HarnessInput<'_>, ppp: f32) -> RawInput {
        let mut raw = RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(input.display.0 / ppp, input.display.1 / ppp))),
            time: Some(input.time_s),
            focused: input.foreground,
            ..RawInput::default()
        };
        raw.viewports.insert(
            ViewportId::ROOT,
            ViewportInfo {
                native_pixels_per_point: Some(ppp),
                focused: Some(input.foreground),
                ..ViewportInfo::default()
            },
        );
        self.pointer(input, ppp, &mut raw.events);
        for event in input.events {
            self.event(*event, &mut raw.events);
        }
        raw
    }

    fn pointer(&mut self, input: &HarnessInput<'_>, ppp: f32, events: &mut Vec<Event>) {
        if !input.foreground {
            if self.prev_cursor.take().is_some() {
                events.push(Event::PointerGone);
            }
            self.prev_left = false;
            self.prev_right = false;
            return;
        }
        let at = pos2(input.cursor.0 / ppp, input.cursor.1 / ppp);
        if self.prev_cursor != Some(at) {
            events.push(Event::PointerMoved(at));
            self.prev_cursor = Some(at);
        }
        for (down, prev, button) in [
            (input.left_down, &mut self.prev_left, PointerButton::Primary),
            (input.right_down, &mut self.prev_right, PointerButton::Secondary),
        ] {
            if down != *prev {
                events.push(Event::PointerButton { pos: at, button, pressed: down, modifiers: self.modifiers });
                *prev = down;
            }
        }
    }

    fn event(&mut self, event: HarnessEvent, events: &mut Vec<Event>) {
        match event {
            HarnessEvent::Text(c) if !c.is_control() => events.push(Event::Text(c.to_string())),
            HarnessEvent::Text(_) => {}
            HarnessEvent::Wheel(data) if data != 0 => events.push(Event::MouseWheel {
                unit: MouseWheelUnit::Line,
                delta: vec2(0.0, data as f32 / Self::WHEEL_NOTCH),
                phase: TouchPhase::Move,
                source: MouseWheelSource::Wheel,
                modifiers: self.modifiers,
            }),
            HarnessEvent::Wheel(_) => {}
            HarnessEvent::Key { code, down, ctrl, shift } => {
                self.modifiers = Modifiers { ctrl, shift, command: ctrl, ..Modifiers::default() };
                match (Self::key(code), ctrl && down) {
                    (Some(Key::C), true) => events.push(Event::Copy),
                    (Some(Key::X), true) => events.push(Event::Cut),
                    (Some(key), _) => events.push(Event::Key {
                        key,
                        physical_key: None,
                        pressed: down,
                        repeat: false,
                        modifiers: self.modifiers,
                    }),
                    (None, _) => {}
                }
            }
        }
    }

    // the egui key for a Windows virtual key code; the keys text fields, dialogs and the world map use
    fn key(code: i32) -> Option<Key> {
        match code {
            0x08 => Some(Key::Backspace),
            0x09 => Some(Key::Tab),
            0x0D => Some(Key::Enter),
            0x1B => Some(Key::Escape),
            0x20 => Some(Key::Space),
            0x23 => Some(Key::End),
            0x24 => Some(Key::Home),
            0x25 => Some(Key::ArrowLeft),
            0x26 => Some(Key::ArrowUp),
            0x27 => Some(Key::ArrowRight),
            0x28 => Some(Key::ArrowDown),
            0x2E => Some(Key::Delete),
            0x30..=0x39 | 0x41..=0x5A => Key::from_name(&char::from(code as u8).to_string()),
            _ => None,
        }
    }

    fn collect(shapes: &[ClippedShape], ppp: f32, out: &mut PassOutput) {
        for clipped in shapes {
            Self::shape(&clipped.shape, ppp, out);
        }
    }

    fn shape(shape: &Shape, ppp: f32, out: &mut PassOutput) {
        match shape {
            Shape::Vec(shapes) => {
                for shape in shapes {
                    Self::shape(shape, ppp, out);
                }
            }
            Shape::Text(text) => {
                let rect = text.visual_bounding_rect();
                let start = out.texts.len() as u32;
                out.texts.push_str(text.galley.text());
                out.text_rects.push(ShownText {
                    start,
                    len: out.texts.len() as u32 - start,
                    rect: [rect.min.x * ppp, rect.min.y * ppp, rect.max.x * ppp, rect.max.y * ppp],
                });
            }
            _ => {}
        }
    }
}

impl Default for UiHarness {
    fn default() -> Self {
        Self::new()
    }
}
