# S00 renderer 메모리 원인과 CPU 후보 근거

검토 날짜2026-10-03. 읽기 전용 조사이며 측정은 통합 담당의 private Xvfb 자료를 사용한다. 측정 binary SHA256 `0006783276df0485f25bfeec0af805389cb81ac0e35d9b25116243cf9c823dd1`의 idle RSS는 default196156KiB, llvmpipe138204KiB, softpipe247564KiB였다. 각10초 idleCPU0%이지만80MiB 제안 인수를 모두 초과했다. 새 CPU binary나 이후 UI 수정의 성능으로 확대하지 않는다.

llvmpipe mapping RSS는 libLLVM15 57580KiB, anonymous37552KiB, heap13540KiB, swrast9784KiB였다. Mesa 공식 llvmpipe 문서는 LLVM JIT 및 최대32thread software rasterization을 명시한다. LLVM mapping을 산술적으로 빼도80624KiB로 예산 여유는1296KiB이며 이는 renderer 제거 후 성능 예측이 아니다. softpipe는 heap139940KiB로 더 나빴다. 현재 depth/stencil/MSAA 기본값은이미0이므로 해당 설정을 끄는 개선으로 보고할 수 없다.

eframe0.33.3 Renderer는 Glow/Wgpu이며 CPU 선택지가 없다. software Wgpu 역시 LLVM 경로에 도달할 수 있어 가상화 환경의 안전한 메모리 해법으로 확정하지 않는다. egui_software_backend0.0.2 rendereronly + 기존 egui-winit AccessKit bridge가 같은 Rust native UI를 유지하며 GL/LLVM runtime 경로를 제거할 독립 실험 후보다. 실제80MiB·IME·reader·clipboard·DPI·worker종료를 통합 시험하기 전 채택 완료라고 표현하지 않는다.

960x640 RGBA 단일 buffer는2.34375MiB, 두 개4.6875MiB이다. 200%1920x1280은 단일9.375MiB/두 개18.75MiB이며 texture/font/cache/nativebuffer 비용은 별도다. font 최대32MiB 및 bounded texture/geometry/frame 정책을 유지하고 원래UI기능 제거·예산 완화·전체egui0.34upgrade는 해법으로 사용하지 않는다.

근거: docs/validation/accessible-performance/{current,software,softpipe}/result.json 및 accessible-performance-validation.md, local eframe/egui-winit0.33.3 source. 외부primary: https://docs.mesa3d.org/drivers/llvmpipe.html , https://docs.rs/eframe/0.33.3/eframe/enum.Renderer.html . 호환 pin은 software-renderer-pin.json, 실제 구현 범위는 cpu-accessible-host.md.
