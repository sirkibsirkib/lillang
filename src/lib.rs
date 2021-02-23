pub(crate) const WORD_SIZE: usize = std::mem::size_of::<usize>();
pub(crate) type OpArgBuf = [usize; 1];

pub mod bytecode;
pub mod vm;

#[cfg(test)]
mod tests;
