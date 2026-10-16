# S00: 다운로드 도구 누락 안내와 회복 동작

- 상태: implementing
- 분류: feature
- 우선순위: P0
- 요구사항: R04, R05, R08
- 담당/수정 범위: 주 담당 통합; crates/app/src/accessible.rs, catalog.rs, media_workspace.rs
- 선행 작업: 현재 일반 GUI collector 종료와 원본 검토 완료

## 사용자 결과

누락된 yt-dlp와 FFmpeg를 구별하고 안내 옆 설치 버튼으로 기존 도구 화면을 연다. URL·저장 경로·미리보기 생략 초안을 보존한다.

## 계약과 범위

DESIGN의 disabled 사유 및 해결 동작 계약을 따른다. 기존 도구 화면 초기화와 worker 정책을 사용한다. 도구 설치나 다운로드는 자동 시작하지 않는다. 동일한 미리보기 사유만 중복 제거한다. schema·Cargo 의존성 변경은 없다. pending local source의 정확한 회복 표시는 후속 검토가 필요하다.

## 인수 조건

실제 Context 클릭 및 키보드로 도구 화면 이동, 초안 보존, CloseWait·작업 중 차단, 명시적 미리보기 생략에서 불필요한 FFmpeg 요구 없음. 사유·버튼 없는 경우 빈 행을 생성하지 않는다. Windows 최신 whole 및 800/960 확대 실제 캡처와 이동 ACK가 필요하다.

## 검증과 handoff

/tmp 후보의 세 파일 핀을 대조하고 통합했다. 두 actual Context 회귀를 추가했다. 아직 Cargo 컴파일·테스트·최신 UI 검증을 실행하지 않았으므로 완료로 취급하지 않는다. 이전 일반 GUI116개 원본 검토는 수정 전 소스에만 유효하다.

최초 통합 검사는 Clippy 단계에서 회귀 테스트가 bounded_font_tests에 배치되어 Fixture/wait_finished를 찾지 못해 실패했다. 원 로그를 보존했다. 실제 Candidate 테스트 모듈로 이동했고 재검증이 필요하다. 성공으로 보고하지 않는다.

수정 후 `windows-msvc-tool-recovery-corrected-whole-20261012` 원본 fmt.log와 clippy.log는 각각 result0 및 자연 종료/Job0/private desktop 종료를 기록했다. 테스트는 아직 실행 중이므로 전체 통과로 표현하지 않는다. 도구 화면 이동 후 다운로드 화면 복귀와 URL·저장 위치 보존을 실제 native UIA/raster로 확인하는 후보를 병렬 준비 중이다.

원본 corrected test.log에서 actual_missing_tool_recovery_opens_tools_without_changing_download_draft 및 actual_tool_recovery_keyboard_busy_and_explicit_bypass_policy가 두 app harness 모두 ok로 실행됨을 확인했다. native OS 입력·hover·raster는 이 Context 테스트로 증명하지 않는다. 전체 harness 집계와 release 빌드 완료는 아직 대기 중이다.

corrected test.log의 원본 harness43개를 집계했다:1123 passed,0 failed,28 ignored. 원 test 작업은 자연 result0 및 Job0을 기록했다. release는 아직 진행 중이므로 whole완료/최신 installer완료로 주장하지 않는다.

새 native recovery 검증 후보는 필수63화면+확장48+배율5+회복4를 유지하고, URL reachability는 필요할 때 생성된 완전 before/after 쌍만 별도 확인하도록 독립 검토기를 수정 중이다. 원 고정126 기대는 최신 optional 계약과 충돌하여 채택하지 않는다. 기존 원본 실패·검증기 기록을 보존한다. 실제 native 실행 및 pixel 인수는 아직 남아 있다.

Windows recovery observer v2의 실제 C# 컴파일을 별도 격리 host에서 실행해 result0, 자연 종료/Job0/private desktop 종료를 확인했다. evidence windows-msvc-recovery-observer-compile-20261012/result.json의 상태는 PASS_COMPILE_ONLY다. 기존 S10 보조 field 두 개의 CS0649 경고는 원 로그에 보존했고 경고 없는 빌드라고 표현하지 않는다. 앱 GUI/회복 동작/pixel 인수는 실행하지 않았다.
