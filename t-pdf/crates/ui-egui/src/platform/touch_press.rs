//! One press per finger. A phone's browser reports a finger coming down twice: as a pointer event,
//! which eframe hands to egui at once in a frame of its own, and then as a touch, which brings a
//! second press of the button already down. Everything that acts on a press saw two: a menu title
//! opened on the first and closed on the second, so the menu bar never opened on a phone (TitanOS
//! PLAN §87). The touch's press is dropped while the button is already down; the touch itself
//! stays, for pinching and for telling a finger's press from the mouse's.

use egui::{Event, PointerButton, RawInput, TouchPhase};

/// Drop this frame's first primary press when a finger came down and the primary button is already
/// down (`primary_down`, from the frame before).
pub(crate) fn one_press_per_touch(raw: &mut RawInput, primary_down: bool) {
    if !primary_down || !raw.events.iter().any(|e| matches!(e, Event::Touch { phase: TouchPhase::Start, .. })) {
        return;
    }
    let mut dropped = false;
    raw.events.retain(|e| {
        let press = matches!(e, Event::PointerButton { button: PointerButton::Primary, pressed: true, .. });
        let keep = dropped || !press;
        dropped |= press;
        keep
    });
}

#[cfg(test)]
mod tests {
    use egui::{Modifiers, TouchDeviceId, TouchId, pos2};

    use super::*;

    fn press(pressed: bool) -> Event {
        Event::PointerButton { pos: pos2(10.0, 10.0), button: PointerButton::Primary, pressed, modifiers: Modifiers::NONE }
    }

    fn touch(phase: TouchPhase) -> Event {
        Event::Touch { device_id: TouchDeviceId(0), id: TouchId(1), phase, pos: pos2(10.0, 10.0), force: None }
    }

    fn input(events: Vec<Event>) -> RawInput {
        RawInput { events, ..Default::default() }
    }

    #[test]
    fn a_touch_after_its_pointer_press_brings_no_second_press() {
        let mut raw = input(vec![press(true), touch(TouchPhase::Start)]);
        one_press_per_touch(&mut raw, true);
        assert_eq!(raw.events, vec![touch(TouchPhase::Start)]);
    }

    #[test]
    fn a_touch_with_nothing_down_keeps_its_press() {
        // A browser without pointer events: the touch is the only press.
        let mut raw = input(vec![press(true), touch(TouchPhase::Start)]);
        one_press_per_touch(&mut raw, false);
        assert_eq!(raw.events.len(), 2);
    }

    #[test]
    fn presses_without_a_touch_are_left_alone() {
        let mut raw = input(vec![press(true), press(false), press(true)]);
        one_press_per_touch(&mut raw, true);
        assert_eq!(raw.events.len(), 3, "the mouse's own presses");
        let mut raw = input(vec![touch(TouchPhase::Move), press(true)]);
        one_press_per_touch(&mut raw, true);
        assert_eq!(raw.events.len(), 2, "only a finger coming down");
    }

    #[test]
    fn only_one_press_goes_and_releases_stay() {
        let mut raw = input(vec![press(true), press(false), touch(TouchPhase::Start), press(true)]);
        one_press_per_touch(&mut raw, true);
        assert_eq!(raw.events, vec![press(false), touch(TouchPhase::Start), press(true)]);
    }
}
