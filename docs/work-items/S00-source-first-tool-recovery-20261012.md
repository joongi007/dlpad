# S00: 영상 입력과 도구 회복의 순서

- 상태: completed
- 분류: UX
- 우선순위: P0
- 요구사항: R04, R05, R08
- 담당/수정 범위: root — crates/app/src/accessible.rs, 이 문서
- 선행 작업: 최신 Windows 전체 검사와 독립 원본 대조 종료

## 사용자 결과

빈 URL 또는 유효하지 않은 URL에서는 영상 입력 안내를 먼저 제공한다. 유효한 원격 URL 또는 로컬 영상이 있을 때 필요한 도구 설치 동작을 제공한다. 상단 도구 탐색은 항상 기존대로 접근한다.

## 계약과 범위

PRODUCT.md의 URL부터 저장까지 흐름, DESIGN.md의 점진 공개, ADR-0007을 따른다. 도구 감지·설치 정책이나 다운로드 gate를 바꾸지 않는다. 이유 문구의 공개 및 시작 조건 확인은 유지한다. 같은 파일은 root만 수정하고 두 감사 에이전트는 /tmp에만 결과를 기록한다.

## 인수 조건

실제 Candidate의 빈/잘못된 URL에서 설치 버튼이 없고 입력 이유가 유지된다. 유효 URL에서 필요한 설치 버튼이 나타나며 클릭 후 URL/저장 초안이 보존된다. 로컬·명시적 미리보기 생략·busy 동작은 기존 계약을 유지한다. 실제 격리 Windows 캡처를 별도 확인한다.

## 검증과 handoff

최신 Light 800×640/앱200% first 캡처를 root가 직접 확인했다. 빈 URL 이유와 yt-dlp·FFmpeg 설치 버튼이 동시에 보였다. 최신 whole 43337 및 독립91460은 종료0이며1157/0/30이다. 아직 이 작업의 변경·회귀·native 시각 인수 완료는 아니다.

- RED64869 exit101: 실제 Candidate 빈 URL에서 설치 버튼 부재 assertion 실패. compiler/수집기 오류가 아니며 원 source 불변, 원 Job 자연 종료와 private desktop 종료를 확인했다.
- 원 SourceUrl parser로 유효한 원격 주소 또는 local source인 경우에만 contextual missing-tool 동작을 표시하도록 수정했다. 시작 이유와 도구 탐색은 유지한다. GREEN33209에서 기존 actual UI 회귀들을 실행 중이며 아직 성공으로 기록하지 않는다.
- 두 독립 감사는 /tmp/dlpad-ui-input-audit-20261012 및 /tmp/dlpad-ui-visual-audit-20261012에 기록했다. 입력 오류 의미 연결, multiline focus outline, 일반 helper/차단 이유 위계는 후속 인수 대상이다. label 공개 위험은 실제 현재 GUI 재현 전이므로 결함 확정으로 쓰지 않는다.
- GREEN33209 exit0: 두 UI harness 각각 actual 회귀52성공/0실패/3ignored. 빈/잘못된 URL의 설치 동작 부재, 유효 URL의 기존 설치 화면 이동·초안 보존, busy/bypass와 시작 이유 스크롤 회귀 통과. source 불변 및 원 Job 자연 종료 확인. native raster 인수는 아니다.
- 차단 이유를 일반 foreground로, 자동 확인/드롭 helper를 small muted로 변경해 중요 정보의 위계를 맞췄다. 원 문구·ID·gate는 유지한다. 이 추가 시각 변경은 새 전체 검사와 native 캡처 인수 대상이다.
- 첫 전체 실행은 상대 output 경로가 fresh evidence 계약을 충족하지 않아 시작 전에 거절됐다. 제품/테스트 실패로 계산하지 않는다. 절대 프로젝트 evidence 경로로 새 실행했다. 설치 ZIP은 아직 이전 후보이며 이 변경이 포함됐다고 주장하지 않는다.
- 새 전체82075와 독립86696 exit0: fmt/clippy/test/release 및 원로그/source/artifact 대조,43 harness/1157성공/0실패/30ignored. CPU9,708,544바이트 SHA `76f54874c7b7636e1ecd5d52d9175b0edd588d57f20e531abd0bddcddbc3f641`; 이전 fresh-publication 대비512바이트 증가. 의존성·lock·schema 변경 없고 SourceUrl의 bounded parser를 기존 UI frame에서 재사용한다. 정확한 최신 UI 비용 표본은 별도다.
- 원 소스를 고정한 채 Light16911/Dark53583 private GUI를 시작했다. first/restart에서 빈 URL의 설치 Button3종 부재·도구 탐색 유지·원 이유 description·이유 위치 공개·후속 휠·자연 종료를 실제 검증한다. 아직 native 성공이나 전체 UI/UX 완료로 기록하지 않는다.

- Light16911/Dark53583 collector exit0: 실제 source-first provider 검사와 원 이유 공개·후속 휠·재시작·테마·자연 종료/Job0 통과. root는 두 theme first/restart 공개 PNG4장을 직접 확인하고 root-scoped-pixel-review.json을 보존했다.
- 초기 독립13140/47420은 수집하지 않은 collector 자체 pin을 요구해 실패했다. 실패/원본 producer/결과를 보존했다. 정정한 독립20384/18751 exit0: 실제 original observer/CPU/whole/current source/raw2/COM/provider/DB/first-restart 보호를 유지하고 self pin 미수집은 false로 기록했다. 각 evidence에 root-independent-native-review.json을 복사했다. 전체 픽셀·물리 입력·전체 UI/UX 인수는 아니다.
- 이 작업의 좁은 인수 조건은 완료했다. 문서36078은939문서 통과했으나 이 마지막 기록 이후의 재검사는 필요하다. 남은 입력 오류 및 multiline focus는 별도 work-item으로 이어가며 설치 ZIP은 아직 이전 후보다.
