# v21 안내 갱신 전 README Windows checkpoint

이 문서는 기존 안내의 보존본이며 현재 설치 경로를 안내하지 않는다. 보존본의 문서 이동에 맞춰 상대 링크만 보정했다.

## Windows UI 검수용 설치

1. [검증된 v18 설치 ZIP](../../docs/validation/evidence/dlpad-install-ui-cross-surface-v18-20261013.zip)을 새 폴더에 모두 압축 해제합니다.
2. dlpad-install.exe를 실행하고 기본 설치 위치를 쓰거나 바꾼 뒤 **설치**를 누릅니다.
3. **설치 확인 완료**에서 **앱 실행**을 누릅니다. 앱이 시작되면 설치 창은 닫히며, 앱 종료까지 원 감독 프로세스가 수명을 관리합니다.

설치 없이 보려면 [v18 앱 실행 파일](../../docs/validation/evidence/ui-test-keyboard-diagnostic-help-20261013/dlpad.exe)을 사용합니다. 기본 저장 폴더와 폴더 선택, 앱 내 도구 설치·선택, Windows 수신·송신 관측, 영상 미리보기·구간 선택, 설명과 실행 계획을 제공하는 고급 옵션, S10 P0 로컬 진단을 포함합니다.

v18에는 공통 글꼴·여백·선택 표현, 네트워크 입력 오류와 첫 관측 준비 표시, 설치 완료 안내·실행, 확대 화면 스크롤과 채워진 버튼 포커스 대비 수정을 포함합니다. 설치 ZIP은 **16,741,557바이트**, SHA-256은 `9fdddac622b22b5bba58d06d30abf7475e0b9e50f47e1d0014f0f03cb2428d07`입니다. [압축·파일 검토](../../docs/validation/evidence/windows-msvc-ui-cross-surface-v18-20261013-package-root-review.json).

이 파일은 서명되지 않은 개발 검수 후보입니다. 기록·도구 목록을 갱신해도 원래 버튼의 대상과 포커스를 유지하도록 수정했습니다. [사용 안내](../../docs/development/windows-user-preview.md), [앱·설치 검수표](../../docs/validation/UI-UX-AUDIT-CURRENT-20261013.md).

## 검증 상태와 남은 작업

v18 Windows fmt·strict clippy·전체 테스트·release 네 명령을 실제 실행했고 **1,327 성공 / 0 실패 / 33 ignored**, 43 harness·source657·원로그/실행 파일/lock을 독립 대조했습니다. ignored된 private GDI 세 시험은 동일 설치기 소스의 별도 격리 실행에서 3/0/0으로 확인한 범위이며 기본 합계에 더하지 않습니다. [전체 검사](../../docs/validation/evidence/windows-msvc-keyboard-diagnostic-help-whole-20261013/root-review.json).

목록 갱신 후 기존 B 명령이 A 작업을 재시작하는 실제 오류와 도구 버튼 ID 변경을 재현·수정했습니다. 기록 8개·도구 17개 관련 시험이 통과했고, 800px 메타데이터 행의 원래 32px 줄바꿈도 보존했습니다. 포커스 보존은 실제 접근성 Focus 요청의 응답이며 물리 키보드 시험과 구분합니다. 목록 identity 변경 당시 앱은 v16보다 2,048바이트 증가했고 설치기·CLI·의존성은 그대로입니다. [목록 회귀](../../docs/validation/evidence/windows-msvc-entity-action-identities-green-v2-20261013/root-review-v2.json).

v18 앱 63개 화면을 팀이 직접 검수했습니다. 최신 일반 설치·실행과 한 바이트 변조 거부는 바깥 실행도 각각0으로 끝났고, private Job0와 원파일·기록 보존을 대조했습니다. 앱·감독 프로세스는 정상 시험에서 자연0, 변조 시험의 설치기는 자연1입니다. 현재 native ID27개도 독립 재조회했습니다. Dark200 하단의 명령 다섯 개는 실제 스크롤 캡처에서 모두 보입니다. [63개 화면](../../docs/validation/evidence/current-keyboard-diagnostic-help-app63-20261013/root-team-visual-review.json), [설치·실행](../../docs/validation/evidence/windows-msvc-ui-cross-surface-v18-20261013-command-happy/root-review.json), [변조 거부](../../docs/validation/evidence/windows-msvc-ui-cross-surface-v18-20261013-command-negative/root-review.json), [ID 대조](../../docs/validation/evidence/windows-msvc-ui-cross-surface-v18-20261013-command-independent/root-review.json), [설치 픽셀](../../docs/validation/evidence/windows-msvc-ui-cross-surface-v18-20261013-command-happy/root-direct-pixel-review.json).

v13 로컬 CI·비용·577/598 UTF-16 긴 경로 설치는 당시 소스의 별도 증거입니다. 긴 경로 앱 실행은 미검증입니다. v14 CI는 외부143 종료와 최종 기록 부재를 미완료로 보존합니다. 비용 strict18의17통과/1실패도 유지하며 원인은 미확인입니다. 전체 P0·물리 IME/reader/OS DPI·clean OS·upgrade/rollback·서명·사용자 최종 UI/UX 평가는 남아 있습니다. Git 조회 exit128이며 기존.git은 보존했고, Git/hosted CI 연결과 updater는 후속입니다. [남은 작업](../../docs/validation/P0-remaining-20261011.md), [검증 이력](../../DEVELOPMENT_HISTORY.md).

