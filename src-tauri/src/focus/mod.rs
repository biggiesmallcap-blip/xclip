use serde::Deserialize;

#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct FocusTools {
    pub enabled: bool,
    pub mode: String,
    pub aspect_ratio: String,
    #[serde(default = "default_shape")]
    pub shape: String,
    pub region: Region,
    pub keyframes: Vec<FocusKeyframe>,
    pub label: Option<LabelOpt>,
    pub style: Option<StyleOpt>,
}

fn default_shape() -> String { "rectangle".to_string() }

#[derive(Deserialize, Clone, Debug)]
pub struct Region {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

#[derive(Deserialize, Clone, Debug)]
pub struct FocusKeyframe {
    pub time: f64,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct LabelOpt {
    pub text: String,
    pub position: String,
    pub text_color: String,
    pub background_color: String,
}

#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct StyleOpt {
    pub stroke_color: String,
    pub stroke_width: u32,
    pub fill_color: String,
    pub fill_opacity: f64,
    pub blur_amount: u32,
    pub pixel_size: u32,
    pub spotlight_dim: f64,
}

const MAX_EXPORT_KEYFRAMES: usize = 30;

/// Sorted timeline (by time, ascending) of effective keyframes used for filter expressions.
/// Falls back to a single keyframe at t=0 from the static region when none are provided.
/// Decimates densely-recorded paths to MAX_EXPORT_KEYFRAMES to keep ffmpeg expressions tractable.
pub fn timeline(focus: &FocusTools, start_time: f64) -> Vec<FocusKeyframe> {
    let mut kfs: Vec<FocusKeyframe> = if focus.keyframes.is_empty() {
        vec![FocusKeyframe {
            time: 0.0,
            x: focus.region.x,
            y: focus.region.y,
            width: focus.region.width,
            height: focus.region.height,
        }]
    } else {
        focus
            .keyframes
            .iter()
            .map(|k| FocusKeyframe {
                time: (k.time - start_time).max(0.0),
                x: k.x,
                y: k.y,
                width: k.width,
                height: k.height,
            })
            .collect()
    };
    kfs.sort_by(|a, b| a.time.partial_cmp(&b.time).unwrap_or(std::cmp::Ordering::Equal));
    decimate(kfs, MAX_EXPORT_KEYFRAMES)
}

fn decimate(kfs: Vec<FocusKeyframe>, max: usize) -> Vec<FocusKeyframe> {
    if kfs.len() <= max || max < 2 { return kfs; }
    let n = kfs.len();
    let mut out: Vec<FocusKeyframe> = Vec::with_capacity(max);
    out.push(kfs[0].clone());
    for i in 1..(max - 1) {
        let idx = (i * (n - 1)) / (max - 1);
        out.push(kfs[idx].clone());
    }
    out.push(kfs[n - 1].clone());
    out.dedup_by(|a, b| (a.time - b.time).abs() < 1e-6);
    out
}

/// Build a piecewise-linear expression for one keyframe field, evaluated at `t` (filter timestamp seconds).
/// Holds the first/last value outside the keyframe range.
pub fn piecewise_expr(kfs: &[FocusKeyframe], field: impl Fn(&FocusKeyframe) -> f64) -> String {
    if kfs.is_empty() {
        return "0".to_string();
    }
    if kfs.len() == 1 {
        return fmt_num(field(&kfs[0]));
    }
    // Build nested if() expressions: if(lt(t,T1), seg(0,1), if(lt(t,T2), seg(1,2), ... last_value))
    let last_v = field(&kfs[kfs.len() - 1]);
    let mut expr = fmt_num(last_v);
    for i in (0..kfs.len() - 1).rev() {
        let a = &kfs[i];
        let b = &kfs[i + 1];
        let av = field(a);
        let bv = field(b);
        let span = (b.time - a.time).max(1e-6);
        // Linear: av + (bv-av)*(t-a.time)/span; clamped via if() chain ordering
        let seg = format!(
            "({}+({})*((t-{})/{}))",
            fmt_num(av),
            fmt_num(bv - av),
            fmt_num(a.time),
            fmt_num(span)
        );
        // For times < first keyframe, hold first value
        if i == 0 {
            // Before first keyframe, hold first value
            let before = fmt_num(av);
            expr = format!("if(lt(t,{}),{},if(lt(t,{}),{},{}))",
                fmt_num(a.time), before, fmt_num(b.time), seg, expr);
        } else {
            expr = format!("if(lt(t,{}),{},{})", fmt_num(b.time), seg, expr);
        }
    }
    expr
}

fn fmt_num(n: f64) -> String {
    // Avoid scientific notation, trim trailing zeros.
    let s = format!("{:.4}", n);
    let s = s.trim_end_matches('0').trim_end_matches('.').to_string();
    if s.is_empty() || s == "-" { "0".into() } else { s }
}

/// Plan returned to the export pipeline.
pub struct FilterPlan {
    /// Pre-scale filter chain to apply to the input video stream.
    /// The export pipeline appends global scale/format/etc afterwards.
    pub filter: String,
    /// If Some, this overrides the quality preset's width handling and produces the given output dims.
    /// (Used for reframe to lock the output to the chosen aspect ratio.)
    pub output_size: Option<(u32, u32)>,
}

pub fn parse_color(input: &str) -> String {
    // ffmpeg accepts #rrggbb directly; pass through, default to red on bad input.
    let s = input.trim();
    if s.starts_with('#') && (s.len() == 7 || s.len() == 4) {
        return s.to_string();
    }
    "#ff3b30".to_string()
}

pub fn aspect_dims(aspect: &str, source: (u32, u32), target_width: Option<u32>) -> (u32, u32) {
    let (sw, sh) = source;
    let ratio: f64 = match aspect {
        "1:1"  => 1.0,
        "4:5"  => 4.0 / 5.0,
        "9:16" => 9.0 / 16.0,
        "16:9" => 16.0 / 9.0,
        _ => sw as f64 / sh as f64,
    };
    let w = target_width.unwrap_or(sw);
    // Make even.
    let mut out_w = w;
    let mut out_h = (out_w as f64 / ratio).round() as u32;
    if out_w % 2 == 1 { out_w -= 1; }
    if out_h % 2 == 1 { out_h -= 1; }
    (out_w.max(2), out_h.max(2))
}

/// Wrap a filter-option expression in single quotes so commas inside `if(...)` aren't
/// mistaken for filter-graph separators by the ffmpeg parser.
fn q(e: &str) -> String { format!("'{}'", e) }

pub fn build_filter_plan(
    focus: &FocusTools,
    source: (u32, u32),
    start_time: f64,
    target_width: Option<u32>,
) -> Result<FilterPlan, String> {
    let kfs = timeline(focus, start_time);
    let x_expr = piecewise_expr(&kfs, |k| k.x);
    let y_expr = piecewise_expr(&kfs, |k| k.y);
    let w_expr = piecewise_expr(&kfs, |k| k.width);
    let h_expr = piecewise_expr(&kfs, |k| k.height);
    let qx = q(&x_expr);
    let qy = q(&y_expr);
    let qw = q(&w_expr);
    let qh = q(&h_expr);

    // For modes that need a fixed-size sub-region (blur/pixelate/spotlight), use the first
    // keyframe's size. drawbox/crop with varying size in overlay is tricky; constant size
    // is more reliable for v1.
    let first_w = kfs.first().map(|k| k.width).unwrap_or(focus.region.width).max(2.0);
    let first_h = kfs.first().map(|k| k.height).unwrap_or(focus.region.height).max(2.0);
    // libx264 with yuv420p requires even dimensions; round down to even (min 2).
    let first_w_int = (((first_w.round() as i64).max(2)) & !1).max(2);
    let first_h_int = (((first_h.round() as i64).max(2)) & !1).max(2);

    let style = focus.style.clone().unwrap_or(StyleOpt {
        stroke_color: "#ff3b30".into(),
        stroke_width: 4,
        fill_color: "#ff3b30".into(),
        fill_opacity: 0.0,
        blur_amount: 12,
        pixel_size: 16,
        spotlight_dim: 0.6,
    });

    // Reframe always uses a rectangular crop — circle shape is ignored there.
    let is_circle = focus.shape == "circle" && focus.mode != "reframe";

    // Alpha-mask expressions on a [0..first_w_int] × [0..first_h_int] crop, normalized to a unit
    // ellipse so circle/ellipse semantics fall out automatically.
    // Filled disc: alpha=255 inside, 0 outside.
    let disc_alpha = format!(
        "'if(lt(hypot((X-{w}/2)/({w}/2),(Y-{h}/2)/({h}/2)),1),255,0)'",
        w = first_w_int, h = first_h_int
    );
    // Ring outline: alpha=255 in [1 - 2T/W, 1] of the normalized radius.
    let ring_thick_norm = (style.stroke_width.max(1) as f64 * 2.0) / (first_w_int as f64);
    let ring_alpha = format!(
        "'if(between(hypot((X-{w}/2)/({w}/2),(Y-{h}/2)/({h}/2)),{inner:.4},1),255,0)'",
        w = first_w_int, h = first_h_int, inner = (1.0 - ring_thick_norm).max(0.0)
    );

    match focus.mode.as_str() {
        "reframe" => {
            // Lock w/h to the first keyframe's size; animate x,y; scale to aspect-locked output.
            let (out_w, out_h) = aspect_dims(&focus.aspect_ratio, source, target_width);
            // Clamp x,y so crop never leaves source bounds.
            let cx = q(&format!("max(0,min({},{}-{}))", x_expr, source.0, first_w_int));
            let cy = q(&format!("max(0,min({},{}-{}))", y_expr, source.1, first_h_int));
            let filter = format!(
                "crop={}:{}:{}:{},scale={}:{}:flags=lanczos,setsar=1",
                first_w_int, first_h_int, cx, cy, out_w, out_h
            );
            Ok(FilterPlan { filter, output_size: Some((out_w, out_h)) })
        }
        "box" => {
            let color = parse_color(&style.stroke_color);
            if is_circle {
                // Generate a transparent ring sized W×H using a solid color source + geq alpha,
                // then overlay it at the animated x,y. Avoids per-pixel evaluation over the whole frame.
                let cx = q(&format!("max(0,min({},{}-{}))", x_expr, source.0, first_w_int));
                let cy = q(&format!("max(0,min({},{}-{}))", y_expr, source.1, first_h_int));
                let mut graph = String::new();
                if style.fill_opacity > 0.0 {
                    let fill_alpha = ((style.fill_opacity * 255.0).round() as i64).clamp(0, 255);
                    graph.push_str(&format!(
                        "color=c={fill}:s={w}x{h}:d=1,format=rgba,geq=r='r(X,Y)':g='g(X,Y)':b='b(X,Y)':a='if(lt(hypot((X-{w}/2)/({w}/2),(Y-{h}/2)/({h}/2)),1),{fa},0)'[fillshape];",
                        fill = parse_color(&style.fill_color),
                        w = first_w_int, h = first_h_int, fa = fill_alpha,
                    ));
                }
                graph.push_str(&format!(
                    "color=c={col}:s={w}x{h}:d=1,format=rgba,geq=r='r(X,Y)':g='g(X,Y)':b='b(X,Y)':a={ring}[ring];",
                    col = color, w = first_w_int, h = first_h_int, ring = ring_alpha,
                ));
                if style.fill_opacity > 0.0 {
                    graph.push_str(&format!(
                        "[0:v][fillshape]overlay=x={cx}:y={cy}:format=auto[withfill];[withfill][ring]overlay=x={cx}:y={cy}:format=auto",
                        cx = cx, cy = cy
                    ));
                } else {
                    graph.push_str(&format!(
                        "[0:v][ring]overlay=x={cx}:y={cy}:format=auto",
                        cx = cx, cy = cy
                    ));
                }
                return Ok(FilterPlan { filter: graph, output_size: None });
            }
            // Rectangle / square (square = rect with 1:1 aspect locked client-side)
            let mut filter = String::new();
            if style.fill_opacity > 0.0 {
                let fill = parse_color(&style.fill_color);
                filter.push_str(&format!(
                    "drawbox=x={}:y={}:w={}:h={}:color={}@{:.3}:t=fill,",
                    qx, qy, qw, qh, fill, style.fill_opacity
                ));
            }
            filter.push_str(&format!(
                "drawbox=x={}:y={}:w={}:h={}:color={}@1:t={}",
                qx, qy, qw, qh, color, style.stroke_width.max(1)
            ));
            Ok(FilterPlan { filter, output_size: None })
        }
        "label" => {
            let label = focus.label.clone().ok_or_else(|| "Label config missing".to_string())?;
            let stroke = parse_color(&style.stroke_color);
            let text_color = parse_color(&label.text_color);
            let bg_color = parse_color(&label.background_color);
            let safe_text = label.text
                .replace('\\', "\\\\")
                .replace(':', "\\:")
                .replace('\'', "\u{2019}");
            let (tx, ty) = match label.position.as_str() {
                "bottom" => (
                    q(&format!("({})+({})/2-text_w/2", x_expr, w_expr)),
                    q(&format!("({})+({})+8", y_expr, h_expr)),
                ),
                "left" => (
                    q(&format!("({})-text_w-8", x_expr)),
                    q(&format!("({})+({})/2-text_h/2", y_expr, h_expr)),
                ),
                "right" => (
                    q(&format!("({})+({})+8", x_expr, w_expr)),
                    q(&format!("({})+({})/2-text_h/2", y_expr, h_expr)),
                ),
                _ => (
                    q(&format!("({})+({})/2-text_w/2", x_expr, w_expr)),
                    q(&format!("({})-text_h-8", y_expr)),
                ),
            };
            let cx = q(&format!("max(0,min({},{}-{}))", x_expr, source.0, first_w_int));
            let cy = q(&format!("max(0,min({},{}-{}))", y_expr, source.1, first_h_int));

            let outline_chain = if is_circle {
                // Two color sources, masked to the ellipse region, overlaid in turn.
                let mut g = String::new();
                if style.fill_opacity > 0.0 {
                    let fill_alpha = ((style.fill_opacity * 255.0).round() as i64).clamp(0, 255);
                    g.push_str(&format!(
                        "color=c={fill}:s={w}x{h}:d=1,format=rgba,geq=r='r(X,Y)':g='g(X,Y)':b='b(X,Y)':a='if(lt(hypot((X-{w}/2)/({w}/2),(Y-{h}/2)/({h}/2)),1),{fa},0)'[fillshape];",
                        fill = parse_color(&style.fill_color),
                        w = first_w_int, h = first_h_int, fa = fill_alpha,
                    ));
                }
                g.push_str(&format!(
                    "color=c={col}:s={w}x{h}:d=1,format=rgba,geq=r='r(X,Y)':g='g(X,Y)':b='b(X,Y)':a={ring}[ring];",
                    col = stroke, w = first_w_int, h = first_h_int, ring = ring_alpha,
                ));
                if style.fill_opacity > 0.0 {
                    g.push_str(&format!(
                        "[0:v][fillshape]overlay=x={cx}:y={cy}:format=auto[withfill];[withfill][ring]overlay=x={cx}:y={cy}:format=auto[boxed];",
                        cx = cx, cy = cy
                    ));
                } else {
                    g.push_str(&format!(
                        "[0:v][ring]overlay=x={cx}:y={cy}:format=auto[boxed];",
                        cx = cx, cy = cy
                    ));
                }
                g
            } else {
                let mut g = String::new();
                if style.fill_opacity > 0.0 {
                    let fill = parse_color(&style.fill_color);
                    g.push_str(&format!(
                        "[0:v]drawbox=x={}:y={}:w={}:h={}:color={}@{:.3}:t=fill,",
                        qx, qy, qw, qh, fill, style.fill_opacity
                    ));
                } else {
                    g.push_str("[0:v]");
                }
                g.push_str(&format!(
                    "drawbox=x={}:y={}:w={}:h={}:color={}@1:t={}[boxed];",
                    qx, qy, qw, qh, stroke, style.stroke_width.max(1)
                ));
                g
            };

            let filter = format!(
                "{outline_chain}[boxed]drawtext=text='{text}':fontcolor={tc}:box=1:boxcolor={bg}@0.85:boxborderw=4:x={tx}:y={ty}",
                outline_chain = outline_chain,
                text = safe_text, tc = text_color, bg = bg_color,
                tx = tx, ty = ty,
            );
            Ok(FilterPlan { filter, output_size: None })
        }
        "blur" => {
            let blur = style.blur_amount.max(1);
            let cx = q(&format!("max(0,min({},{}-{}))", x_expr, source.0, first_w_int));
            let cy = q(&format!("max(0,min({},{}-{}))", y_expr, source.1, first_h_int));
            let crop_blur = format!(
                "crop={}:{}:{}:{},boxblur={}:1",
                first_w_int, first_h_int, cx, cy, blur
            );
            let filter = if is_circle {
                format!(
                    "split=2[bgmain][bgcopy];[bgcopy]{crop_blur},format=rgba,geq=r='r(X,Y)':g='g(X,Y)':b='b(X,Y)':a={mask}[fx];[bgmain][fx]overlay=x={cx}:y={cy}:format=auto",
                    crop_blur = crop_blur, mask = disc_alpha, cx = cx, cy = cy
                )
            } else {
                format!(
                    "split=2[bgmain][bgcopy];[bgcopy]{crop_blur}[fx];[bgmain][fx]overlay=x={cx}:y={cy}",
                    crop_blur = crop_blur, cx = cx, cy = cy
                )
            };
            Ok(FilterPlan { filter, output_size: None })
        }
        "pixelate" => {
            let n = style.pixel_size.max(2);
            let cx = q(&format!("max(0,min({},{}-{}))", x_expr, source.0, first_w_int));
            let cy = q(&format!("max(0,min({},{}-{}))", y_expr, source.1, first_h_int));
            let small_w = (first_w_int / n as i64).max(1);
            let small_h = (first_h_int / n as i64).max(1);
            let crop_pix = format!(
                "crop={}:{}:{}:{},scale={}:{}:flags=neighbor,scale={}:{}:flags=neighbor",
                first_w_int, first_h_int, cx, cy, small_w, small_h, first_w_int, first_h_int
            );
            let filter = if is_circle {
                format!(
                    "split=2[bgmain][bgcopy];[bgcopy]{crop_pix},format=rgba,geq=r='r(X,Y)':g='g(X,Y)':b='b(X,Y)':a={mask}[fx];[bgmain][fx]overlay=x={cx}:y={cy}:format=auto",
                    crop_pix = crop_pix, mask = disc_alpha, cx = cx, cy = cy
                )
            } else {
                format!(
                    "split=2[bgmain][bgcopy];[bgcopy]{crop_pix}[fx];[bgmain][fx]overlay=x={cx}:y={cy}",
                    crop_pix = crop_pix, cx = cx, cy = cy
                )
            };
            Ok(FilterPlan { filter, output_size: None })
        }
        "spotlight" => {
            let dim = style.spotlight_dim.clamp(0.0, 1.0);
            let cx = q(&format!("max(0,min({},{}-{}))", x_expr, source.0, first_w_int));
            let cy = q(&format!("max(0,min({},{}-{}))", y_expr, source.1, first_h_int));
            let dim_amt = format!("{:.3}", -dim);
            let filter = if is_circle {
                format!(
                    "split=2[bgmain][bgcopy];[bgmain]eq=brightness={dim},boxblur=4:1[bgdim];[bgcopy]crop={w}:{h}:{cx}:{cy},format=rgba,geq=r='r(X,Y)':g='g(X,Y)':b='b(X,Y)':a={mask}[clear];[bgdim][clear]overlay=x={cx}:y={cy}:format=auto",
                    dim = dim_amt, w = first_w_int, h = first_h_int, cx = cx, cy = cy, mask = disc_alpha
                )
            } else {
                format!(
                    "split=2[bgmain][bgcopy];[bgmain]eq=brightness={dim},boxblur=4:1[bgdim];[bgcopy]crop={w}:{h}:{cx}:{cy}[clear];[bgdim][clear]overlay=x={cx}:y={cy}",
                    dim = dim_amt, w = first_w_int, h = first_h_int, cx = cx, cy = cy
                )
            };
            Ok(FilterPlan { filter, output_size: None })
        }
        other => Err(format!("Unsupported focus mode: {other}")),
    }
}
