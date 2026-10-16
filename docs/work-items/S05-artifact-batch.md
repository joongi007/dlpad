# S05: 영상과 자막의 완료 증거 묶음

- 상태: in-progress
- 분류: feature
- 우선순위: P0
- 요구사항: R07, R08
- 담당/수정 범위: 주 담당; crates/core/src/jobs.rs, engine/download.rs·공유 publication schema, app/main.rs·catalog.rs, scripts/subtitle_smoke.py, crates/engine/tests/local_media.rs, 본 문서
- 선행 작업: S05-subtitle typed/helper; 기존 publication 복구

## 사용자 결과

영상과 선택한 한국어·영어 자막을 한 Job의 결과로 관리한다. 필요한 결과의 일부만 확인한 상태를 완료로 표시하지 않는다.

## 계약과 범위

CONTEXT Artifact와 ADR-0002. core는 파일 I/O 없이 검증된 증거 묶음을 원자적으로 수용한다. 최대 영상 1개와 자막 2개, 빈·중복 경로와 0 byte 거절. 기존 단일 complete 호출은 호환 유지. engine의 실제 파일·source 검증과 publication/recovery journal 연결은 별도로 수행하며 core만으로 UI 자막 완료를 주장하지 않는다. 공유 schema는 주 담당만 조정한다.

## 인수 조건

Verifying 상태에서만 전체 묶음을 받아 한 번의 revision으로 Downloaded가 된다. 잘못된 후속 항목·빈 묶음·중복·상한 초과에는 상태/revision/artifacts 불변. 기존 단일 영상 완료의 동작을 유지한다. 최종 기능에는 실제 sidecar publication과 crash 복구·UI·도구 fixture 시험이 필요하다.

## 검증과 handoff

구현 중. core 회귀 시험 이후 실제 engine 다중 publication을 이어서 연결한다.

### core 구현 증거

complete_many와 단일 complete 위임을 구현했다. cargo test -p dlpad-core --locked: 21개 통과. 빈 묶음·0 byte 후속 항목·중복/빈 경로·4개 상한 초과에서 상태/revision/artifacts 불변, 정상 3개에 한 번의 revision, 완료 뒤 재호출 거절을 검증했다. 실제 engine 다중 파일 공개·복구와 자막 UI는 아직 연결하지 않아 작업은 in-progress다.

### publication journal 연결

adapter migration v5에 nullable planned_artifacts_json(128KiB 상한)을 추가했다. 최대 3개 staging/final/bytes/Unix dev+ino 계획을 파일 공개 전에 저장한다. 전체 staging과 기존 final 충돌을 먼저 검사하고 create-only hardlink를 적용한다. 크래시 복구는 native identity가 있는 경우 누락된 final을 공개하고 전체 묶음을 완료한다. legacy 단일 계획은 기존 복구를 유지한다. 비Unix native identity는 아직 없어 batch 복구는 fail-closed이며 사용자 파일은 보존한다. cleanup은 DB 완료 이후 matching identity의 staging link만 대상으로 한다. raw path same-user TOCTOU와 실제 자막 metadata/UI 연결은 남는다.

### 실제 실행 검증

cargo test -p dlpad-engine download:: --locked: 41개 통과·공식 네트워크 opt-in 1개 ignored(필터 문자열에 direct_download 포함). SQLite를 닫고 다시 열어 일부 공개된 영상/한·영 자막 묶음 복구와 손상 JSON 보존을 검증했다. cargo build -p dlpad --bin dlpad --release --locked 성공(38.03초); 이 바이너리를 관리 도구 GUI 담당에게 전달했다. 첫 engine clippy는 불필요 unit 표현 1개 실패로 해당 표현을 수정했다. core+engine 정책 구현이며 실제 yt-dlp 자막 metadata/UI 연결은 여전히 남는다.

### 실제 자막 증거 엔진 연결

yt-dlp after_video projection을 id·한/영 filepath/ext로 한정하고 source URL이나 signed 자막 URL을 출력하지 않는다. 실제 기존 yt-dlp 2026.09.27.232945 합성 loopback smoke에서 missing 언어는 빈 object라는 점을 실증해 helper를 조정했다. duplicate key·언어 수 상한은 빈/null에도 적용한다. selected source와 owned staging 검증 뒤 영상+자막을 v5 publication 묶음에 넣었다. synthetic executable의 3개 결과·자막 없음·잘못된 경로 실패 시험 1개 통과(3.69초). actual tool projection smoke 1개 통과; 공식 관리 pin 2026.08.19 및 actual engine 미디어+자막 통합·UI는 아직 남는다.

### 자막 UI 연결

저장 안 함/한국어/영어/한·영 선택과 자동 생성 허용을 중앙 문자열·typed 옵션에 연결했다. 초기값은 저장 안 함이며 실행 중 변경은 차단한다. 실행 계획·저장 snapshot에 선택을 반영하고 구간+자막 충돌은 기존 옵션 검증으로 시작을 거절한다. 첫 UI 시험은 충돌이 options() 단계에서 검증된다는 점을 잘못 가정한 테스트 unwrap이 실패해, 실제 UI 계약에 맞게 오류를 단언하도록 수정했다. 제품 정책은 완화하지 않았다. 가상 화면 자막 조작/actual engine media+subtitle 검증은 아직 남는다.

### 전체 회귀와 실제 media 통합 시험

자막 UI 연결 뒤 cargo test --workspace --locked: app14/core21/engine115/diagnostic3/guardian8 및별도fixture1 통과, 공식네트워크/실제도구opt-in2개 ignored. 당시 Windows 전용시험은 Linux에서0개이며 성공으로계산하지 않는다. 기존 local_media opt-in에 actual yt-dlp 요청/영상+VTT 결과/DB reopen 검증을 추가했다. 테스트 전용 Python argv adapter는 임시 info-json을 공급하고 os.execv로 실제 도구를 실행하며 제품 옵션/config 정책을 변경하지 않는다. 실제 사이트 extractor 지원을 증명하는 시험은 아니다.

DLPAD_SMOKE_YTDLP=<existing tool> DLPAD_SMOKE_FFMPEG=<existing ffmpeg> cargo test -p dlpad-engine --test local_media --locked -- --ignored: 1개 실제 도구 통합 시험 통과(36.55초). 기존 전체/빠른·정확 구간 decode와 새 영상+한국어 VTT publication/DB reopen이 포함됐다. source fixture의 extractor 입력은 test adapter가 제공하므로 실제 YouTube·치지직 지원 검증은 별도다.

### 격리 GUI 독립 검토

자막 담당은 동일release SHA7e3afa82764e4bc52b254f337db39c81431efede22c7dce9b3bb2aa8aafdfa3d에서 저장안함·ko·en·both·구간충돌·실행중freeze 및ko auto-off를 실제Xvfb로 검증했다. 주 담당은 보존JSON의오류없음과구간충돌PNG의한국어오류/도움말을직접 확인했다. freeze는중단뒤두번째GUIStart의options/snapshot도비교하여 단순첫Job snapshot보존과구분한다. 실제사이트/3OS GUI와원본언어/구간자막기능은남는다. 새 download_soak 짧은엔진시험을 주 담당이직접실행해1통과/8h opt-in1ignored(6.11초)를확인했다.
