# S00: 다운로드 화면 시각 밀도 개선

- 상태: implementation-verification
- 분류: ui-quality
- 우선순위: P0
- 요구사항: R01, R07, R08
- 담당/수정 범위: 주 담당 accessible.rs/catalog.rs/통합; diagnostic_cache media_workspace.rs 후보; diagnostic_inputs accessible.rs 후보; 하위 담당은 /tmp만 수정
- 선행 작업: native source freeze 종료 및 원본 guard 검토(충족); 최신 전체 검증 진행 중

## 사용자 결과

빈 미리보기의 비중과 좁은 창의 중복 여백을 줄여 저장 설정 접근을 개선한다. 오류 전문과 준비된 영상 표시는 유지한다.

## 계약과 범위

Rust 네이티브와 DESIGN/기존 ADR 유지. schema/Cargo/worker 변경 없음. 프로젝트 수정은 주 담당만 수행.

## 인수 조건

800/960 Light/Dark 실제 전후 캡처, 긴 오류 문구 가시성, ready 영상 aspect-fit 유지,32px 입력/폴더 선택/배율/스크롤/포커스 회귀 유지. 실제 hover/pressed는 별도 관측하며 static PNG로 대신하지 않는다.

## 검증과 handoff


상태: 감사 완료, 구현 대기(실행 중 GUI source freeze 이후).

범위: R01/R07/R08 다운로드 기본 흐름의 중복 안내·빈 미리보기 비중·좁은 화면 저장 설정 간격 개선. 주 담당이 accessible.rs, media_workspace.rs, catalog.rs 및 본 문서를 담당한다. 의존성: 최신 GUI collector terminal 및 원본 guard 검토. worker/저장/네트워크/schema/Cargo 변경 없음.

인수 조건: 800/960 Light/Dark 직접 캡처 비교; 도구 누락 원인 안내 한곳으로 통합하되 필수 원인·도구 접근 유지; empty/blocked preview만 compact 처리, ready 영상 aspect-fit 유지; 입력32px/폴더 선택/배율/스크롤/포커스 기존 인수 유지. hover·pressed pixels는 별도 실제 관측한다.

감사 근거: diagnostic_cache가 최신 first PNG6개와 DESIGN/GTmux Input·Button 토큰을 비교했다. 주 담당도 download-full960/settings960 및200%800/960를 직접 보았다. 중복 tool 안내와 빈 영역의 비중을 다음 개선 대상으로 채택했다. 전체 UI 완료 또는 사용자 시각 승인으로 주장하지 않는다.

구현 검토: 다운로드 disabled reason의 yt-dlp 부재와 미리보기 error의 FFmpeg 포함 원인은 서로 다른 조건이다. 두 번째 안내를 무조건 숨기면 skip-preview 또는 FFmpeg-only 부재에서 원인을 잃을 수 있다. compact preview도 실제 오류의 여러 줄을 잘라서는 안 된다. diagnostic_cache는 media_workspace.rs 후보만 /tmp에서 담당하며 주 담당은 accessible.rs/catalog 및 통합을 맡는다. 실행 중 native source는 변경하지 않았다.

구현 통합: 모든 source-frozen native 시험 종료 및 raw guard/current source 검토 후 원본SHA를 확인하고 두 후보를 주 담당이 통합했다. idle/unprepared preview만96..120 최소 높이(긴 오류 자연 성장), ready/busy 기존140..200/aspect-fit 유지. narrow16 explicit spacer와 중복 separator margin 제거, 필요한 local/range/bypass prelude간격 유지. 원인 문구 삭제 없음. 검증 전이며 이 변경 이후 기존 whole/ZIP/GUI는 변경 전 빌드의 증거다.

최신 전체4검사 완료: windows-msvc-download-visual-density-whole-20261012/root-review.json에서 raw Job guards/current623 source/originals/artifact pins와 exact4command 계약을 주 담당이 독립 확인했다.43 harness/1,119 passed/0 failed/28 ignored. CPU9,672,704 bytes SHA256 f78117cedbb73602ee4e158ae7f5b6fbe677729bfffc297843730ecc1eae0fa7. 새 current-ui-download-visual-density-20261012 가상 화면 캡처 시작. 실제시각 인수·이 빌드 설치 ZIP 갱신은 아직이다.

새800 Light 실제 PNG 직접 검토: URL/저장폴더 함께 노출, preview120px 및 두줄 도구 오류 전문 가시성 확인. preview하단341→separator374 약32px 여백이 남아 제안16px와 다르다. show_media_preview 뒤 빈 horizontal_wrapped 등 실제layout 추가확인 대상으로 기록한다. 품질은 이 위치의 viewport 아래이며 접근 가능성과 동일화하지 않는다. Dark/ready/hover는 미검증.

Latest200%800/960 배율 캡처를 주 담당이 직접 검토: label/help/control 전체가 footer 위에 있다. density 변경 뒤 해당 조건의 clipping 회귀는 관찰되지 않는다. root-density-pixel-review.json에 PNG SHA를 보존했다. 빈 playback horizontal_wrapped guard 후보는 별도 검토했고 source-frozen GUI 종료 후 통합 예정이다.

최신GUI collector 종료0. 첫 독립 reviewer는 과거69固定inventory의 추가reachability6PNG를 강제해 FAIL했다. 원본 collector는63required와 실제필요시150/200 before/after쌍을허용한다. 최신density에서추가쌍0/전체116=63+48+5를확인했으며 새readonlyreview는 모든required/48/5와boundedoptionalcompletepairs를유지해검토한다. 실패기록을삭제하지 않고 UIpixel인수와별도로기록한다.

독립 raw review PASS: current source/pins/private cleanup와required63+48+5=116 실제PNG를확인했다. 직접시각검토범위는별도유지한다. 모든native sourcefreeze종료후원본SHA대조하여빈playback행guard를통합했다. prepared/busy에서만행을생성하며ready/busy/cancel/error/timeline정책그대로. 변경후재검증필요; 이전whole/GUI/ZIP은이수정전증거이다.

Empty playback row 수정 전체검증완료: root-review.json 원본logs/currentsource/pins/GNUoriginals/artifacts 확인.43harness1119/0/28,CPU9,672,704B SHA518d944f5b7b866a48975fbfbe4e855fd11c9c788f050d651d2e61934259d3e1. 새16px여백native캡처는아직필요하다. 네트워크관측UI의Linux전용input은별도요구사항감사로분리한다.

### 최신 화면 직접 검토 — 2026-10-12

진행 중인 `current-ui-installer-network-final-20261012`의 `initial-download-full-960.png`를 직접 열어 확인했다. SHA-256: `016366963682556c49687fc1a35c86d101b1c316d20de191ca6c95caed12e24d`. URL 입력 및 폴더 선택은 보이나, 긴 yt-dlp 회복 안내에 직접 도구 이동 동작이 없고 왼쪽 미리보기 아래 빈 공간과 오른쪽 저장 설정의 footer 앞 잘림이 시각적으로 불균형하다. 이 캡처는 중간 관측이며 collector 전체 성공·호버·실제 마우스 인수를 증명하지 않는다. 도구 회복 동작 후보는 별도 /tmp에서 준비하며 실행 중 source는 변경하지 않는다. UI/UX 완료 상태는 유지하지 않는다.

### 저장 inspector 폭 후보 검토

주 담당이 `/tmp/dlpad-download-inspector-geometry/candidate.patch`를 읽었다. 현재 300px frame의 내부268px는 folder_field inline 기준360px에 못 미쳐 browse를 별도32px행으로 배치한다. 400px frame 내부368px 후보는 기존 inline 정책을 활용하며 기능·옵션을 제거하지 않는다. 현재는 제안 상태이고 프로젝트에 적용하지 않았다. 760px wide 분기에서 남는 preview 폭, 실제 ready/busy/error 및 확대800/960 화면을 확인해야 한다. geometry 변경 시 기존 native reviewer의 download960 width268/stack 기대도 새 실제계약에 맞춰 갱신해야 하며 단순 통과를 위해 완화하지 않는다.

### 저장 설정 배치 후보 통합

기존 wide inspector300→400px를 적용했다. 기존folder_field360px inline 조건을 활용하며 옵션·도움말·scroll을 제거하지 않았다. 실제 준비된 합성 영상/긴경로/바쁜준비/실패를 검사하는 ignored Context 회귀를 기존 Candidate 테스트 모듈에 넣었다. 이 시험은 소유fixture/FFmpeg 공급 후 명시적으로 실행해야 하며 아직 컴파일·실행되지 않았다. 이전 whole1123/root-review는 수정 전 source 증거다. 새로운 build 및 실제 ready raster가 필요하다.

실제 ready ignored 시험은 정확1개 실행됐으나 폴더 선택 Button의 unique assertion에서2개로 실패했다(result101). 원본 windows-msvc-ready-inspector-geometry-20261012/test.log를 보존했다. 이전 incremental 접근성 노드가 map에 남는 가설을 검사 중이며 unique 조건을 첫 항목 선택으로 완화하지 않는다. 시험 실패 상태이므로 actual ready geometry 완료를 주장하지 않는다.

현재 AccessKit root.children에 도달 가능한 노드만 유지하는 test 수집 수정을 통합했다. warmup/ready/focus/error update에 적용하고 cycle·깊이256·노드10000 상한을 검사한다. unique browse/geometry 인수 조건은 유지했다. 수정 후 실제 native 실행 전이므로 이전 실패 원인을 확정하거나 성공으로 승격하지 않는다.

최신 network-reveal-finalpass whole의 원본4검사/currentsource/실행파일/GNU핀 검토가 완료됐다:1129 passed,0failed,30ignored,harness43. 이후 actual ready 시험은 정확1개실행에서 prepared texture paint를 찾지 못해 실패했다. 이전 browse unique assertion은 통과했지만 전체 geometry 인수는 실패 상태다. 실제 frame decode 시점과 Image의 epaint 표현을 조사하며 private texture 주입/조건삭제로 통과시키지 않는다. 원 failure result/log를 보존했다.

실제 pinned egui0.33.3 image.rs:851-886의 paint_texture_at를 읽어 rotationNone에서 RectShape.with_texture를 사용함을 확인했다. 제품 Image는 회전하지 않아 Mesh만 찾던 test helper가 실제영상shape를누락했다. nonfont texture RectShape도 검사하도록 수정했다. 크기/clip/비겹침 조건은 유지하며 실제 재실행 전 성공으로 취급하지 않는다.

최신 viewport-fit 시험은 800px/200%에서 이미지 y89..199, body y61..171로 실패했다. 준비 완료 상태만으로 3 frame을 세던 회귀를 실제 이미지 전체 paint가 안정된 3 frame으로 변경했다. 초기 공개 스크롤 적용 지연과 지속적인 레이아웃 잘림을 구분하며 20초 제한과 최소160×90 조건은 유지한다. 재실행 결과는 아직이다.

viewport-settled 회귀는 20초 뒤 전체 이미지 안정 조건으로 실패했다(1 failed, exit101, 소유 Job 종료·입력 desktop 보존). 준비 프레임의 내부 top_down/add_space/Image 배치를 exact canvas allocation과 중앙 image paint로 변경해 widget 간격이 canvas 높이를 늘리지 않게 했다. 최소 이미지 크기와 clip 인수 조건은 유지한다.

정확한 canvas 진단에서 안정 image y62.5..172.5/body y61..171로 1.5px 하단 초과를 확인했다(800/200%, exit101). clip 경계와의 일치를 피하도록 body 높이에서8 논리px 여유를 확보했다. 최소160×90·whole paint 검증은 그대로 유지. 문서 검사 실제 PASS:909 documents/114 upstream notices. 독립 에이전트 읽기 검토는 동일frame 공개만 반복됨을 확인했지만 native/wheel 성공을 주장하지 않았다.

canvas-inset 실제 Windows CPU 회귀 PASS: accessible::tests::actual_ready_preview_and_inspector_geometry_across_widths 정확1 passed/0 failed/0 ignored, exit0. 960/824 100%,800 150%/200% actual decoded texture full clip·최소160×90·폴더 focus 전체노출·missing source error glyph 전체노출·원본fixture보존을 검증했다. 원로그의 Job natural-zero/final-zero/final-active0/private desktop closed/input desktop unchanged 확인. Context paint 검증이며 실제 native screenshot 및 wheel/hover 인수는 아직이다.
