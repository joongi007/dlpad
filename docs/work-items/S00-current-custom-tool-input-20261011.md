# S00: 현재 custom 도구 경로 입력 검증

- 상태: completed (현재 private 입력 인수 범위)
- 분류: verification
- 우선순위: P0
- 요구사항: R04, R05, R07, R13
- 담당/수정 범위: root native 실행·통합 및 scripts 후보; diagnostic_inputs는 /tmp observer 후보만 준비한다. 공유 Rust/Cargo/schema/lock 변경 없음.
- 선행 작업: 입력별 인수표·현재 Windows whole proof

## 사용자 결과

yt-dlp·FFmpeg·Deno 경로 입력의 실제 편집, 유효성 표시와 화면 이동 뒤 readback을 좁은 사례로 확인한다.

## 계약과 범위

별도 private desktop과 합성 실행 파일만 사용한다. 사용자 Documents·실제 계정·미디어·외부 네트워크를 사용하지 않는다. 물리 포인터·IME와 private provider action을 구분한다. 현재 수동 경로의 영구 저장 계약이 없을 가능성은 문서·실험으로 확인하며 저장 성공을 추정하지 않는다.

## 인수 조건

세 입력의 absolute·Unicode/긴 경로·잘못된 경로 사례 및 실제 policy 오류를 source와 raw 결과에 연결한다. native original process/Job 종료, 현재 SHA, readback 및 화면 증거를 root가 검토한다. 후보 작성만으로 완료 처리하지 않는다.

## 검증과 handoff

초기 범위 기록. 프로젝트 변경과 실행은 root가 순서대로 진행한다.

첫 native 실행은 UIA worker의 180초 시험 deadline을 초과해 FAIL했다. 첫 yt-dlp absolute/relative/복원 및 다음 FFmpeg absolute까지 일부 readback이 있으나 전수 PASS가 아니다. worker drain cleanup과 원 desktop 보존을 기록했다. 한글 긴 문자열의 fallback WM_CHAR를 글자마다 150ms와 실제 소유 확인으로 입력하므로 수백 글자·다중 사례는 이 시험 budget을 넘는다. 두 번째 별도 후보는 제품/입력/판정 코드를 바꾸지 않고 worker600초·외부720초의 bounded deadline으로 실행한다. 기존 실패 evidence와 원 후보 SHA를 보존한다.

두 번째 native 실행은9개경로 readback 후 URL 입력으로 이동할 때 helper가 offscreen URL의 focus를 먼저 요청하지 않아 bounded wheel 탐색에서 FAIL했다. 실패 PNG·raw와 forced owned cleanup을 보존했다. 제품 공유 입력 helper는 gained_focus시 reveal을 이미 수행한다. 세 번째 후보는 도구 입력과 동일하게 URL의 exact provider focus를 먼저 요청하며, 제품 source 변경이나 wheel/IME 성공 주장은 하지 않는다.

세 번째 실제 실행을 root가 통합 인수했다. [root 검토](../validation/evidence/windows-msvc-current-custom-tool-input-third-20261011/root-review.json):3도구9개한글/긴absolute·relative·복원 readback32px, 화면 이동 유지,3relativeplan거절/복원, FFmpeg/Deno exact literalargv, 같은data 재실행 초기DLPAD환경값 복귀를 확인했다. 두 GUI process의 original HANDLE/creation·natural exit0/Job0·private desktop/input원상복구와 현재whole/source/CPU/helper/합성tool/sentinel SHA를 검토했다. Root가3개복원 PNG의 실제glyph를 직접 확인했다. 수동 경로는 재실행에 저장되지 않으며 이것을 저장 성공으로 표현하지 않는다. 실제 외부 tool version/다운로드 효과·물리pointer/IME·managedUse충돌·multiline전수 인수는 남는다. 검증된 재현 helper2개를 scripts에 통합했고 /tmp pycache로 실제 구문 검사를 통과했다. Git090595d source snapshot은 이 최종 helper/인수 문서 추가 전이며 과거snapshot으로 보존한다.
