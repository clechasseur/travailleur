use crate::workflow::definition::events::EventKind;

pub fn if_not_used_for_compensation_then_must_have_transition_or_end<'t, 'u, T, U, C>(
    transition: &'t Option<T>,
    end: &'u Option<U>,
) -> impl FnOnce(&bool, &C) -> garde::Result + 't + 'u
where
    't: 'u,
    'u: 't,
    C: ?Sized,
{
    |used_for_compensation, _ctx| {
        if !used_for_compensation && transition.is_none() && end.is_none() {
            Err(garde::Error::new(""))
        } else {
            Ok(())
        }
    }
}

pub fn mandatory_for_consumed_events<C>(
    kind: EventKind,
) -> impl FnOnce(&Option<String>, &C) -> garde::Result
where
    C: ?Sized,
{
    move |source, _ctx| {
        if kind == EventKind::Consumed && source.is_none() {
            Err(garde::Error::new("event source is mandatory for consumed events"))
        } else {
            Ok(())
        }
    }
}
