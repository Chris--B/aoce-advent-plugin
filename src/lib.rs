use pyo3::intern;
use pyo3::prelude::*;

use log::*;

#[pyclass]
struct Answer {
    #[pyo3(get)]
    input_data: String,

    #[pyo3(get)]
    answer_a: String,

    #[pyo3(get)]
    answer_b: String,

    #[pyo3(get)]
    extra: PyObject,
}

#[pymodule]
fn aoce_advent_plugin(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(advent_example_parser, m)?)?;
    Ok(())
}

#[pyfunction]
fn advent_example_parser(
    py: Python<'_>,
    page: PyObject,
    _datas: PyObject,
) -> PyResult<Vec<Answer>> {
    let html = page.getattr(py, intern!(py, "raw_html"))?;
    let html: &str = html.extract(py)?;

    let year: u32 = page.getattr(py, intern!(py, "year"))?.extract(py)?;
    let day: u32 = page.getattr(py, intern!(py, "day"))?.extract(py)?;

    if let Some(input_data) = extract_example(year, day, html) {
        Ok(vec![Answer {
            input_data,
            answer_a: "".into(),
            answer_b: "".into(),
            extra: py.None(),
        }])
    } else {
        Ok(vec![])
    }
}

pub fn extract_example(year: u32, day: u32, html: &str) -> Option<String> {
    // Note: year and day are unused atm
    // We can use them to hard-code tricky days, but I'd rather get the heuristics working first.

    let dom = tl::parse(html, tl::ParserOptions::default()).unwrap();
    let parser = dom.parser();

    let codes: Vec<_> = dom
        .nodes()
        .iter()
        .filter_map(tl::Node::as_tag)
        .filter(|n| n.name().as_bytes() == b"code")
        .filter(|n| n.inner_html(parser).lines().count() > 1)
        .collect();

    fn fixup(s: &str) -> Option<String> {
        // TODO: Can I do this in lt? Maybe if we didn't ask for "inner html"?
        let s = s.replace("&gt;", ">");
        let s = s.replace("&lt;", "<");
        let s = s.replace("<em>", "");
        let s = s.replace("</em>", "");
        let s = s.replace("<code>", "");
        let s = s.replace("</code>", "");

        // None of the input should have trailing empty lines
        let s = s.trim_end_matches('\n').to_string();

        Some(s)
    }

    // Other ideas:
    //      - Fetch input first and pick first codeblock that uses the same alphabet?

    match codes.len() {
        0 => {
            // Do nothing and fall through to the error handling below.
        }
        1 => {
            debug!("Only found a single code block, using that directly");

            // Only one option, so we should return it
            return fixup(&codes[0].inner_html(parser));
        }
        n_codes => {
            debug!("Found {n_codes} <code>, trying to narrow them down");

            // Most examples are prefixed with "For example", or something similar.
            // So find the *last* node that contains that text (its parents show up first),
            // and then return the first code block after that
            if let Some(search_start) = dom.nodes().iter().rposition(|n| {
                n.inner_html(parser)
                    .to_ascii_lowercase()
                    .contains("for example")
            }) {
                // Find the first <span> after "for example"
                if let Some(node) = dom
                    .nodes()
                    .iter()
                    .skip(search_start)
                    .take(5) // We expect this immediately after, so don't look too far
                    .filter_map(tl::Node::as_tag)
                    .find(|t| t.name().as_bytes() == b"pre")
                {
                    let inner = node.inner_html(parser);
                    if inner.contains("<code>") {
                        debug!("Found <pre> after \"for example\" with <code>, using that");
                        return fixup(&inner);
                    } else {
                        debug!("Found <pre> after \"for example\" but it didn't contain <code>");
                    }
                }

                debug!("Found \"for example\" but couldn't find a <pre><code> block after it");
            } else {
                debug!("Couldn't find \"for example\" at all");
            }

            // We have multiple options to choose from. Most of the time, this
            // is the first. But we can check for some signs that an earlier <code>
            // block is not relevant.
            for code in codes.iter() {
                // 2022/Day 7 https://adventofcode.com/2022/day/7:
                //      Day7 starts with an irrelevant block, but it includes a
                //      non-visible <span>.
                // We can look for tags and reject blocks that include them.
                // Note: < and > are valid characters in some puzzles, so we need to
                // know about HTML for this check.
                // So far, counting children seems to work
                {
                    if code.children().all(parser).len() == 1 {
                        debug!("Found a multi-line <code> with 1 child, using that");
                        return fixup(&code.inner_html(parser));
                    }
                }
            }
        }
    }

    // We got down here without finding anything. Let's look for single-line code blocks too
    {
        for code in dom
            .nodes()
            .iter()
            .filter_map(tl::Node::as_tag)
            .filter(|n| n.name().as_bytes() == b"code")
            .filter(|n| n.inner_html(parser).lines().count() <= 1)
        {
            debug!("Found a single-line <code> with 1 child, using that");
            if code.children().all(parser).len() == 1 {
                return fixup(&code.inner_html(parser));
            }
        }
    }

    // If we get down here, all of our heuristics failed to ID an example
    // That's kinda bad. :(
    // So complain loudly
    let n_blocks = codes.len();
    if n_blocks == 0 {
        error!("🚧 Failed to find an example for AoC {year} day{day}. We couldn't find any code blocks to check 🚧");
    } else {
        error!("🚧 Failed to find an example for AoC {year} day{day}. We found {n_blocks} <code>, and none worked. 🚧");
    }

    None
}
