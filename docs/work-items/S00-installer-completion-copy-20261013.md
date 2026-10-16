# S00: 설치 완료 단계와 다음 행동 안내

- 상태: completed
- 분류: UX
- 우선순위: P1
- 요구사항: R08, R11
- 담당/수정 범위: root — crates/app/src/installer_main.rs, installer_catalog.rs 및 본 기록. installer_batch는 /tmp 후보와 격리 화면 수집기, advanced_units는 /tmp 패키지 준비, windows_auto는 /tmp 최종 CI 준비만 담당한다.
- 선행 작업: v11 설치 실행 성공·변조 거부 인수 및 선택 글자 대비 통합 검증 완료.

## 사용자 결과

설치·재검증을 마치면 ‘설치 확인 완료’와 앱 실행 안내를 보여준다. 경로 변경이나 실패에는 준비 안내로 돌아간다.

## 계약과 범위

기존 verified 상태에서 중앙 catalog의 완료 제목·설명을 고르고 기존 STATIC 컨트롤을 갱신한다. 실행 admission 실패가 verified를 유지하는 기존 재시도 정책은 바꾸지 않고 안내만 준비로 돌린다. 기존 버튼, 파일·Job 소유권, 실패 receipt, schema와 Cargo는 유지한다. 창이 닫히는 시점은 원 process-start ACK이며 GUI 준비 handshake가 아니다.

## 인수 조건

실제 격리 Windows 설치의 초기/검증 완료/새 경로 편집/원 경로 복귀·재검증/실행 상태에서 제목·설명·Run 활성화를 확인한다. 명시적 변조 거부에서 준비 안내·원 실패 보존과 자식 미생성을 확인한다. 전체 fmt/clippy/test/release, 원본 source/파일 ID/7 source/5 payload/journal 보존 및 캡처를 root가 독립 검토한다.

## 검증과 handoff

v11 실제 캡처에서 검증 완료 뒤에도 ‘설치 준비’와 설치 위치 입력 지시가 남음을 확인했다. /tmp/dlpad-installer-completion-stage-copy-candidate-20261013에 2파일 후보를 보존했다. 단순 문구 selector를 그대로 반복하는 4개 테스트는 채택하지 않으며 기존 상태 회귀와 실제 native 단계 전이를 검증한다. 구현·검증·패키지 인수는 진행 중이다.

## v13 실제 완료 안내와 복귀 인수

최신 Windows 전체 fmt/clippy/test/release는1,297성공/0실패/30ignored,656소스 항목에 묶여 있다. v13 설치의 초기 준비→설치·재검증 완료→새 소유 sibling 경로 편집으로 준비/실행 비활성화→원 경로 복귀·재검증으로 완료/실행 활성화→앱 실행 수명을 실제 격리 화면에서 확인했다. 편집한 새 target/journal은 끝까지 생성하지 않았고 원본7파일·설치5파일·journal 및 원Job/프로세스 identity를 보존했다. Light100 완료·경로 편집과 Dark200 완료 PNG를 Root가 직접 보아 한글 누락이나 제목·설명 겹침이 없음을 국소 확인했다. [단계 전이 검토](../validation/evidence/windows-msvc-ui-cross-surface-v13-20261013-completion-happy-v3/root-review.json), [직접 화면 검토](../validation/evidence/windows-msvc-ui-cross-surface-v13-20261013-completion-happy-v3/root-visual-review.json).

변조 거부에서도 준비 안내 복귀와 설치/검증 활성화·앱 실행 비활성화를 실제 확인했고 Root가 실패 PNG를 보았다. 원Job의 생성 프로세스 수1은 유지됐으며 실패 receipt 보존으로 설치기는 자연exit1이었다. [거부 인수](../validation/evidence/windows-msvc-ui-cross-surface-v13-20261013-completion-negative-v3/root-review.json), [실패 화면](../validation/evidence/windows-msvc-ui-cross-surface-v13-20261013-completion-negative-v3/root-visual-review.json).

원 v1 CS0170 초기화 실패와 v2 remote HFONT 선택 실패는 원로그/helper/FAIL 그대로 유지한다. v3는 관측한 LOGFONT로 observer 소유 font/DC를 만들어 높이를 근사 측정하고 자기 객체만 복구·해제한다. `metadata_clone_height_fit=true`, `original_font_measurement_verified=false`이며 다른 프로세스 HFONT를 선택·삭제하지 않는다. 이 근사 결과와 Root의 실제 PNG 검수를 구분한다. 전체 UI/물리입력/OS DPI·reader·서명·정식릴리스 인수는 미완료다. 단순 selector를 반복하는4시험은 채택하지 않았으며 실제 단계 전이로 이 문구 수정 범위를 인수했다.
