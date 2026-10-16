# S00: Windows 파일명 이모지 표시 복구

- 상태: in-progress
- 분류: UX
- 우선순위: P0
- 요구사항: R03, R08
- 담당/수정 범위: root — crates/app/src/accessible.rs, 실제 검증과 이 문서; diagnostic_cache — /tmp 읽기 조사 및 후보만
- 선행 작업: 파일 목록 전체 경로 창의 실제 Windows 캡처

## 사용자 결과

파일명과 전체 경로의 지원 가능한 이모지가 누락 글리프 네모로 표시되지 않는다.

## 계약과 범위

DESIGN의 OS 글꼴 및 bounded startup I/O 계약을 유지한다. 추가 의존성 없이 설치된 Windows 글꼴의 outline glyph 지원을 먼저 확인한다. DLPAD_TEST_FONT의 단일 fixture 의미를 보존한다. 색상 emoji 및 ZWJ 전체 지원을 단일 glyph 결과로 주장하지 않는다. 현재 GUI 실행이 종료되기 전 제품 source를 변경하지 않는다.

## 인수 조건

기존 한글/영문과 입력 기하를 보존하며 🙂 단일 glyph의 실제 Context 및 Windows 캡처를 확인한다. 없는 글꼴은 기존 fallback을 유지하고 파일 크기 상한을 적용한다. 추가 시작 읽기·메모리 비용을 기록한다.

## 검증과 handoff

첫 Light800200 경로 상세 캡처에서 실제 한글은 표시되고 🙂는 네모로 보였다. Windows 설치 글꼴 seguiemj.ttf 12,934,920B, seguisym.ttf 2,514,056B를 읽기 확인했다. 아직 제품 추가·glyph 지원·시각 인수 완료가 아니다.

- 모든 기존 native/독립 handle 종료 후 root가 후보 diff를 검토해 Segoe UI Symbol optional preload와 실제 Context glyph/outline mesh 회귀를 통합했다. 기존 UI/한글 순서와 테스트 override 의미를 유지하며 파일당32MiB 상한을 유지한다. 추가 raw2,514,056B, 실제 세 글꼴 합16,933,004B; 최악 aggregate 상한96MiB이며 parsed/atlas/시작 latency는 미측정이다. Cargo/의존성 변경 없음. 아직 컴파일 및 native glyph 인수 완료는 아니다.

- 진행 중 whole57080의 원 test.log에서 actual_windows_symbol_font_rasterizes_smile_without_replacement가 기본/CPU 각각 ok임을 root가 읽었다. 실제 설치 Symbol 글꼴로 proportional/monospace glyph 및 outline mesh를 검증했다. 전체 명령은 아직 terminal이 아니며 새 native 픽셀/시작 latency/컬러·ZWJ 인수로 확대하지 않는다.

- 전체57080 및 독립11889 exit0: 1203성공/0실패/30ignored. CPU9,780,224B/SHA `b900fffdbdb618670ef6f9ac53479232c2aa10b8f72c7085d9197fb5b75678cc`. 기존 선택 상태 whole보다512B증가. 새 native glyph 인수는 다음 안내 배치 수정과 함께 수행한다.
