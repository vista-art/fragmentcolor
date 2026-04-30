// pcg — PCG random integer hash, 1D → 1D. Pass the output to successive calls
// to step an RNG, or just use it as a stateless hash.
fn pcg(n: u32) -> u32 {
  var state = n * 747796405u + 2891336453u;
  let word = ((state >> ((state >> 28u) + 4u)) ^ state) * 277803737u;
  return (word >> 22u) ^ word;
}
