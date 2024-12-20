use rustler::{Encoder, Env, NifResult, Term};

pub fn encode_response<'a, T>(
    env: Env<'a>,
    status: rustler::types::atom::Atom,
    source: rustler::types::atom::Atom,
    message: T,
) -> NifResult<Term<'a>>
where
    T: Encoder,
{
    Ok((status, source, message).encode(env))
}
