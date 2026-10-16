# 0008: Windows 선택 도구 설치와 활성 경로 복원

- 상태: Accepted (Windows 개발 후보의 구현 범위)
- 관련: R05, R07, R08, R11; ADR 0003, 0005

## 배경

archive 검증 엔진은 있었지만 앱은 Deno와 FFmpeg 설치를 막고 있었다. 사용자가 앱 내 설치와 설치 프로그램의 선택 설치를 요구했다. 기존 사용자 도구와 앱 설치 소유권은 구분한다.

## 결정

Windows x64에서 고정된 HTTPS URL·길이·SHA-256·구성원·라이선스 manifest를 기존 worker에 연결한다. yt-dlp direct 후보와 Deno/FFmpeg archive 후보는 중앙 managed_candidates로 공유한다. archive 설치 완료는 활성 선택과 다르며 도구 화면의 선택·사용으로 실제 버전 검증 후 활성화한다.

설치 프로그램의 세 선택 항목은 기본 해제한다. 앱 prepare 성공 후 같은 worker에서 순차 설치하고 전체 절대 300초 deadline과 원 취소를 공유한다. 각 도구는 남은 시간과 기존 최대 120초 중 작은 한도를 따른다. 실패 시 이후 도구를 시작하지 않는다. 앱 성공과 도구 공급 결과를 별도로 표시하며 기존 활성 도구를 보존한다. 도구 data root는 source/bundle/journal과 겹치지 않는지 mutation 전에 검사한다.

앱 시작 시 원 복구 worker가 기존 owned root를 검사한다. 호환·무결성·실제 버전 검증을 통과한 활성 선택을 우선하고, 활성 선택이 없거나 잘못됐으면 정확한 현재 고정 후보의 기존 설치만 검사한다. 임의 버전 전수 실행이나 자동 다운로드는 수행하지 않는다. 검사 자체는 active를 쓰지 않으며 복원/설치 연결 단계가 저장한 Automatic·시작 환경값 부재·명시 입력 revision·닫기 조건을 충족할 때만 검증된 후보를 활성화한다. Manual·빈 값을 포함한 환경 override·사용자 수정은 보존한다.

개발용 --tool-data 절대 경로는 설치 프로그램의 시험 프로필이다. 자동 선택을 만들지 않으며 실제 GUI 시험은 앱과 같은 소유 TEMP data root를 사용한다. 일반 설치는 앱 기본 data root를 사용한다.

## 비용과 검증 범위

Cargo 의존성·feature·lock 변경 없이 기존 HTTPS/TLS·archive 코드를 installer에 처음 링크해 installer가 1,103,360B에서 4,290,560B로 증가했다. worker 없는 idle 앱에는 자동 네트워크 작업을 추가하지 않는다. 실제 Windows 도구 공급·재실행과 private desktop 시험은 작업 문서의 결과를 기준으로 한다. 공개 릴리스의 서명·법적 완결·clean OS 검증을 이 결정으로 대신하지 않는다.

Windows 접근성 adapter 0.29.2에는 ExpandCollapse provider가 없어 expanded 정보만으로 설정을 조작할 수 없다. 펼침 상태와 동등한 Toggle 정보를 함께 제공하고 기존 egui Click 경로를 사용한다. 다른 플랫폼의 expanded 의미는 유지한다.

## 2026-10-11 수동 선택 저장 보강

root가 별도 preference controller를 연결했다. 설정 key에 고정3개 Manual/Automatic snapshot을 기존 worker Store로 저장하며 DB schema/Cargo/lock은 변경하지 않는다. 환경 override(빈값 포함)·새로운 사용자 path/mode revision·active 저장 완료·CloseWait ACK를 분리한다. 명시적 managed Use는 그 slot만 Automatic으로 저장한다. malformed row의 자동 복구·외부 도구 소유권 변경·GUI yt-dlp PATH 자동발견은 포함하지 않는다. 실제 Windows CPU223passed/0failed/3ignored이며 전체 workspace와 실제 UI 저장/재실행 인수는 진행 중이다. 구현 결정 채택과 정식 릴리스 인수를 구분한다.

## 2026-10-13 검사·비활성 설치·자동 선택 결과 연결

Windows 실제 격리 시험의 RED/진단/회귀를 근거로 읽기 전용 inspector와 명시적 verified selector가 동일 10초/원 cancel/hash/runtime/native identity 검증을 공유하도록 구현했다. direct yt-dlp install-only API는 active SQL을 실행하지 않으며, 기존 활성 설치 API는 transaction의 commit 이전에 취소를 확인한다. 각 검사 기한을 전체 도구 batch의 10초 보장으로 확대하지 않는다. Windows DOS prefix 검사로 명백한 비실행 파일의 native 시작 지연을 피하지만 full PE/publisher 검증으로 표현하지 않는다.

자동 설치 dispatch는 Controller가 만든 슬롯별 opaque path/mode revision ticket을 전달하고 실제 결과는 그 ticket을 그대로 반환한다. 현재 saved Automatic·환경 override 부재·revision 일치·CloseWait 조건을 다시 확인할 때만 session 경로를 반영한다. background 검사나 결과는 새 preference 저장을 만들지 않는다. 명시 Use의 기존 한 슬롯 저장은 유지한다. 검사 실패 row는 이유와 함께 비활성화하고, 검사하지 않은 이전 설치는 `검사 후 사용`으로 실제 검증을 거쳐 선택한다.

현재 8개 생산 파일을 채택했으며 Windows 전체 fmt/clippy/test/release 재검증 중이다. actual 격리 CPU366/0/4, native verifier 및 취소2/0, install-only 및 UPDATE 이후 취소2/0은 좁은 근거다. 실제 GUI 자동 설치·새 최신 버전 공급·업데이트 UI·물리 입력·배포 인수를 대신하지 않는다. Cargo/dependency/schema/상주 worker 추가는 없고, 경로 cache 중복은 최대3×32768 bytes 및 렌더 중 한 slot 경로 복사 상한이 추가된다. 정확한 PE/runtime 비용은 새 전체 빌드 이후 측정한다.

## 2026-10-13 v20 통합 검증 완료 범위

Windows MSVC `windows-msvc-managed-tool-auto-whole-20261013`의 fmt/clippy/workspace test/release build는 모두 통과했다(1357 PASS/0 FAIL/37 ignored). root가 whole 원본 로그·657 source map·현재 실행파일 3개의 정확한 SHA/bytes를 독립 확인했다. v20 앱 63 PNG는 root11+하위52 분담 직접 검수 및 현재 image pin 일치를 확인했고, installer 주요8 PNG와 정상/음수 launch raw lifetime/source/input pin도 확인했다. 새 다운로드 환경 설명 배치는 별도 probe geometry 오차가 있어 미인수다. P1 최신 조회/동적 공급/UI 통합은 후보이며 이 설치본에 포함되지 않는다. 전체 P0·물리 IME·정식 서명/공개 배포 완료를 뜻하지 않는다.
