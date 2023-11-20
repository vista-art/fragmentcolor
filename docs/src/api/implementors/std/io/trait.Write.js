(function() {var implementors = {
"arrayvec":[["impl&lt;const CAP: <a class=\"primitive\" href=\"https://doc.rust-lang.org/1.73.0/std/primitive.usize.html\">usize</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.73.0/std/io/trait.Write.html\" title=\"trait std::io::Write\">Write</a> for <a class=\"struct\" href=\"arrayvec/struct.ArrayVec.html\" title=\"struct arrayvec::ArrayVec\">ArrayVec</a>&lt;<a class=\"primitive\" href=\"https://doc.rust-lang.org/1.73.0/std/primitive.u8.html\">u8</a>, CAP&gt;"]],
"base64":[["impl&lt;W: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.73.0/std/io/trait.Write.html\" title=\"trait std::io::Write\">Write</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.73.0/std/io/trait.Write.html\" title=\"trait std::io::Write\">Write</a> for <a class=\"struct\" href=\"base64/write/struct.EncoderWriter.html\" title=\"struct base64::write::EncoderWriter\">EncoderWriter</a>&lt;W&gt;"],["impl&lt;S: StrConsumer&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.73.0/std/io/trait.Write.html\" title=\"trait std::io::Write\">Write</a> for <a class=\"struct\" href=\"base64/write/struct.EncoderStringWriter.html\" title=\"struct base64::write::EncoderStringWriter\">EncoderStringWriter</a>&lt;S&gt;"]],
"env_logger":[["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.73.0/std/io/trait.Write.html\" title=\"trait std::io::Write\">Write</a> for <a class=\"struct\" href=\"env_logger/fmt/struct.Formatter.html\" title=\"struct env_logger::fmt::Formatter\">Formatter</a>"]],
"flate2":[["impl&lt;R: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.73.0/std/io/trait.Read.html\" title=\"trait std::io::Read\">Read</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.73.0/std/io/trait.Write.html\" title=\"trait std::io::Write\">Write</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.73.0/std/io/trait.Write.html\" title=\"trait std::io::Write\">Write</a> for <a class=\"struct\" href=\"flate2/read/struct.GzEncoder.html\" title=\"struct flate2::read::GzEncoder\">GzEncoder</a>&lt;R&gt;"],["impl&lt;W: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.73.0/std/io/trait.BufRead.html\" title=\"trait std::io::BufRead\">BufRead</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.73.0/std/io/trait.Write.html\" title=\"trait std::io::Write\">Write</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.73.0/std/io/trait.Write.html\" title=\"trait std::io::Write\">Write</a> for <a class=\"struct\" href=\"flate2/bufread/struct.DeflateDecoder.html\" title=\"struct flate2::bufread::DeflateDecoder\">DeflateDecoder</a>&lt;W&gt;"],["impl&lt;W: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.73.0/std/io/trait.Read.html\" title=\"trait std::io::Read\">Read</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.73.0/std/io/trait.Write.html\" title=\"trait std::io::Write\">Write</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.73.0/std/io/trait.Write.html\" title=\"trait std::io::Write\">Write</a> for <a class=\"struct\" href=\"flate2/read/struct.DeflateEncoder.html\" title=\"struct flate2::read::DeflateEncoder\">DeflateEncoder</a>&lt;W&gt;"],["impl&lt;W: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.73.0/std/io/trait.Write.html\" title=\"trait std::io::Write\">Write</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.73.0/std/io/trait.Write.html\" title=\"trait std::io::Write\">Write</a> for <a class=\"struct\" href=\"flate2/write/struct.DeflateDecoder.html\" title=\"struct flate2::write::DeflateDecoder\">DeflateDecoder</a>&lt;W&gt;"],["impl&lt;R: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.73.0/std/io/trait.Read.html\" title=\"trait std::io::Read\">Read</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.73.0/std/io/trait.Write.html\" title=\"trait std::io::Write\">Write</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.73.0/std/io/trait.Write.html\" title=\"trait std::io::Write\">Write</a> for <a class=\"struct\" href=\"flate2/read/struct.MultiGzDecoder.html\" title=\"struct flate2::read::MultiGzDecoder\">MultiGzDecoder</a>&lt;R&gt;"],["impl&lt;W: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.73.0/std/io/trait.Write.html\" title=\"trait std::io::Write\">Write</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.73.0/std/io/trait.Write.html\" title=\"trait std::io::Write\">Write</a> for <a class=\"struct\" href=\"flate2/write/struct.DeflateEncoder.html\" title=\"struct flate2::write::DeflateEncoder\">DeflateEncoder</a>&lt;W&gt;"],["impl&lt;W: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.73.0/std/io/trait.Read.html\" title=\"trait std::io::Read\">Read</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.73.0/std/io/trait.Write.html\" title=\"trait std::io::Write\">Write</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.73.0/std/io/trait.Write.html\" title=\"trait std::io::Write\">Write</a> for <a class=\"struct\" href=\"flate2/read/struct.DeflateDecoder.html\" title=\"struct flate2::read::DeflateDecoder\">DeflateDecoder</a>&lt;W&gt;"],["impl&lt;W: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.73.0/std/io/trait.Write.html\" title=\"trait std::io::Write\">Write</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.73.0/std/io/trait.Write.html\" title=\"trait std::io::Write\">Write</a> for <a class=\"struct\" href=\"flate2/struct.CrcWriter.html\" title=\"struct flate2::CrcWriter\">CrcWriter</a>&lt;W&gt;"],["impl&lt;R: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.73.0/std/io/trait.Read.html\" title=\"trait std::io::Read\">Read</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.73.0/std/io/trait.Write.html\" title=\"trait std::io::Write\">Write</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.73.0/std/io/trait.Write.html\" title=\"trait std::io::Write\">Write</a> for <a class=\"struct\" href=\"flate2/read/struct.ZlibDecoder.html\" title=\"struct flate2::read::ZlibDecoder\">ZlibDecoder</a>&lt;R&gt;"],["impl&lt;R: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.73.0/std/io/trait.BufRead.html\" title=\"trait std::io::BufRead\">BufRead</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.73.0/std/io/trait.Write.html\" title=\"trait std::io::Write\">Write</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.73.0/std/io/trait.Write.html\" title=\"trait std::io::Write\">Write</a> for <a class=\"struct\" href=\"flate2/bufread/struct.GzDecoder.html\" title=\"struct flate2::bufread::GzDecoder\">GzDecoder</a>&lt;R&gt;"],["impl&lt;W: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.73.0/std/io/trait.Write.html\" title=\"trait std::io::Write\">Write</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.73.0/std/io/trait.Write.html\" title=\"trait std::io::Write\">Write</a> for <a class=\"struct\" href=\"flate2/write/struct.GzEncoder.html\" title=\"struct flate2::write::GzEncoder\">GzEncoder</a>&lt;W&gt;"],["impl&lt;W: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.73.0/std/io/trait.Write.html\" title=\"trait std::io::Write\">Write</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.73.0/std/io/trait.Write.html\" title=\"trait std::io::Write\">Write</a> for <a class=\"struct\" href=\"flate2/write/struct.ZlibDecoder.html\" title=\"struct flate2::write::ZlibDecoder\">ZlibDecoder</a>&lt;W&gt;"],["impl&lt;W: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.73.0/std/io/trait.Read.html\" title=\"trait std::io::Read\">Read</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.73.0/std/io/trait.Write.html\" title=\"trait std::io::Write\">Write</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.73.0/std/io/trait.Write.html\" title=\"trait std::io::Write\">Write</a> for <a class=\"struct\" href=\"flate2/read/struct.ZlibEncoder.html\" title=\"struct flate2::read::ZlibEncoder\">ZlibEncoder</a>&lt;W&gt;"],["impl&lt;R: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.73.0/std/io/trait.BufRead.html\" title=\"trait std::io::BufRead\">BufRead</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.73.0/std/io/trait.Write.html\" title=\"trait std::io::Write\">Write</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.73.0/std/io/trait.Write.html\" title=\"trait std::io::Write\">Write</a> for <a class=\"struct\" href=\"flate2/bufread/struct.ZlibDecoder.html\" title=\"struct flate2::bufread::ZlibDecoder\">ZlibDecoder</a>&lt;R&gt;"],["impl&lt;R: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.73.0/std/io/trait.BufRead.html\" title=\"trait std::io::BufRead\">BufRead</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.73.0/std/io/trait.Write.html\" title=\"trait std::io::Write\">Write</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.73.0/std/io/trait.Write.html\" title=\"trait std::io::Write\">Write</a> for <a class=\"struct\" href=\"flate2/bufread/struct.GzEncoder.html\" title=\"struct flate2::bufread::GzEncoder\">GzEncoder</a>&lt;R&gt;"],["impl&lt;R: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.73.0/std/io/trait.BufRead.html\" title=\"trait std::io::BufRead\">BufRead</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.73.0/std/io/trait.Write.html\" title=\"trait std::io::Write\">Write</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.73.0/std/io/trait.Write.html\" title=\"trait std::io::Write\">Write</a> for <a class=\"struct\" href=\"flate2/bufread/struct.ZlibEncoder.html\" title=\"struct flate2::bufread::ZlibEncoder\">ZlibEncoder</a>&lt;R&gt;"],["impl&lt;R: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.73.0/std/io/trait.Read.html\" title=\"trait std::io::Read\">Read</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.73.0/std/io/trait.Write.html\" title=\"trait std::io::Write\">Write</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.73.0/std/io/trait.Write.html\" title=\"trait std::io::Write\">Write</a> for <a class=\"struct\" href=\"flate2/read/struct.GzDecoder.html\" title=\"struct flate2::read::GzDecoder\">GzDecoder</a>&lt;R&gt;"],["impl&lt;W: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.73.0/std/io/trait.Write.html\" title=\"trait std::io::Write\">Write</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.73.0/std/io/trait.Write.html\" title=\"trait std::io::Write\">Write</a> for <a class=\"struct\" href=\"flate2/write/struct.MultiGzDecoder.html\" title=\"struct flate2::write::MultiGzDecoder\">MultiGzDecoder</a>&lt;W&gt;"],["impl&lt;W: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.73.0/std/io/trait.BufRead.html\" title=\"trait std::io::BufRead\">BufRead</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.73.0/std/io/trait.Write.html\" title=\"trait std::io::Write\">Write</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.73.0/std/io/trait.Write.html\" title=\"trait std::io::Write\">Write</a> for <a class=\"struct\" href=\"flate2/bufread/struct.DeflateEncoder.html\" title=\"struct flate2::bufread::DeflateEncoder\">DeflateEncoder</a>&lt;W&gt;"],["impl&lt;W: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.73.0/std/io/trait.Write.html\" title=\"trait std::io::Write\">Write</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.73.0/std/io/trait.Write.html\" title=\"trait std::io::Write\">Write</a> for <a class=\"struct\" href=\"flate2/write/struct.GzDecoder.html\" title=\"struct flate2::write::GzDecoder\">GzDecoder</a>&lt;W&gt;"],["impl&lt;W: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.73.0/std/io/trait.Write.html\" title=\"trait std::io::Write\">Write</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.73.0/std/io/trait.Write.html\" title=\"trait std::io::Write\">Write</a> for <a class=\"struct\" href=\"flate2/write/struct.ZlibEncoder.html\" title=\"struct flate2::write::ZlibEncoder\">ZlibEncoder</a>&lt;W&gt;"]],
"objc2":[["impl&lt;T: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.73.0/std/io/trait.Write.html\" title=\"trait std::io::Write\">Write</a> + ?<a class=\"trait\" href=\"https://doc.rust-lang.org/1.73.0/core/marker/trait.Sized.html\" title=\"trait core::marker::Sized\">Sized</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.73.0/std/io/trait.Write.html\" title=\"trait std::io::Write\">Write</a> for <a class=\"struct\" href=\"objc2/rc/struct.Id.html\" title=\"struct objc2::rc::Id\">Id</a>&lt;T, <a class=\"struct\" href=\"objc2/rc/struct.Owned.html\" title=\"struct objc2::rc::Owned\">Owned</a>&gt;"],["impl&lt;T: <a class=\"trait\" href=\"objc2/declare/trait.IvarType.html\" title=\"trait objc2::declare::IvarType\">IvarType</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.73.0/std/io/trait.Write.html\" title=\"trait std::io::Write\">Write</a> for <a class=\"struct\" href=\"objc2/declare/struct.Ivar.html\" title=\"struct objc2::declare::Ivar\">Ivar</a>&lt;T&gt;<span class=\"where fmt-newline\">where\n    &lt;Self as <a class=\"trait\" href=\"https://doc.rust-lang.org/1.73.0/core/ops/deref/trait.Deref.html\" title=\"trait core::ops::deref::Deref\">Deref</a>&gt;::<a class=\"associatedtype\" href=\"https://doc.rust-lang.org/1.73.0/core/ops/deref/trait.Deref.html#associatedtype.Target\" title=\"type core::ops::deref::Deref::Target\">Target</a>: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.73.0/std/io/trait.Write.html\" title=\"trait std::io::Write\">Write</a>,</span>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.73.0/std/io/trait.Write.html\" title=\"trait std::io::Write\">Write</a> for <a class=\"struct\" href=\"objc2/foundation/struct.NSMutableData.html\" title=\"struct objc2::foundation::NSMutableData\">NSMutableData</a>"]],
"png":[["impl&lt;'a, W: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.73.0/std/io/trait.Write.html\" title=\"trait std::io::Write\">Write</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.73.0/std/io/trait.Write.html\" title=\"trait std::io::Write\">Write</a> for <a class=\"struct\" href=\"png/struct.StreamWriter.html\" title=\"struct png::StreamWriter\">StreamWriter</a>&lt;'a, W&gt;"]],
"termcolor":[["impl&lt;'a&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.73.0/std/io/trait.Write.html\" title=\"trait std::io::Write\">Write</a> for <a class=\"struct\" href=\"termcolor/struct.StandardStreamLock.html\" title=\"struct termcolor::StandardStreamLock\">StandardStreamLock</a>&lt;'a&gt;"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.73.0/std/io/trait.Write.html\" title=\"trait std::io::Write\">Write</a> for <a class=\"struct\" href=\"termcolor/struct.Buffer.html\" title=\"struct termcolor::Buffer\">Buffer</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.73.0/std/io/trait.Write.html\" title=\"trait std::io::Write\">Write</a> for <a class=\"struct\" href=\"termcolor/struct.BufferedStandardStream.html\" title=\"struct termcolor::BufferedStandardStream\">BufferedStandardStream</a>"],["impl&lt;W: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.73.0/std/io/trait.Write.html\" title=\"trait std::io::Write\">Write</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.73.0/std/io/trait.Write.html\" title=\"trait std::io::Write\">Write</a> for <a class=\"struct\" href=\"termcolor/struct.Ansi.html\" title=\"struct termcolor::Ansi\">Ansi</a>&lt;W&gt;"],["impl&lt;W: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.73.0/std/io/trait.Write.html\" title=\"trait std::io::Write\">Write</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.73.0/std/io/trait.Write.html\" title=\"trait std::io::Write\">Write</a> for <a class=\"struct\" href=\"termcolor/struct.NoColor.html\" title=\"struct termcolor::NoColor\">NoColor</a>&lt;W&gt;"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.73.0/std/io/trait.Write.html\" title=\"trait std::io::Write\">Write</a> for <a class=\"struct\" href=\"termcolor/struct.StandardStream.html\" title=\"struct termcolor::StandardStream\">StandardStream</a>"]]
};if (window.register_implementors) {window.register_implementors(implementors);} else {window.pending_implementors = implementors;}})()