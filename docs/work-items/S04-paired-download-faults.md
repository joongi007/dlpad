# S04: Windows paired 공개 다운로드 실패·취소 fixture

- 상태: implemented; native-validation-pending
- 분류: validation
- 우선순위: P0
- 요구사항: R04, R08, R11
- 담당/수정 범위: soak_recovery_audit; examples/windows_paired_download_fixture.rs와 본 문서만
- 선행 작업: paired 공개 성공 fixture/root native 통합

## 사용자 결과

실제 metadata 실패/download 실패/실행 중 취소에서도 양 receipt·안전 cleanup·재사용과 source/DB privacy를 확인한다.

## 계약과 범위

기존 성공 인수 유지. optional success/metadata-failure/download-failure/cancel mode; fake tool은 CAP/TEMP/private cookie 실제 bytes를 확인한다. cancel은 도구 cookie read 뒤 owned readiness와 bounded poll을 사용한다. synthetic만 사용, parent crash/Cargo/sharedAPI/다른 source 변경 제외. 실패/미확인 정리는 pending을 보존하며 통과로 바꾸지 않는다.

## 인수 조건

실제 원 도구 exit41/42 또는 Cancelled outcome, Failed/Cancelled job·artifact0, auth output omission. mode별 actual stage·receipt수, metadata/download cookiepath, 실제 양 cleanup·managed9files 제거/reuse·source/sentinel/hash 보존 및 DB/WAL/SHM privacy. root 실제 native build/run 전 완료 주장 금지.

## 검증과 handoff

source freeze SHA256 `b166ffafaae16499bd26e920072577dba0501771773bd6b4cc435ea9cf18cac2`. root CLI 계약 `windows_paired_download_fixture.exe --fixture-run <fresh-result.json> [--scenario metadata-failure|download-failure|cancel]`; 옵션 없으면 기존 success. 원 fake 도구는 CAP/mode를 검사하고 실제 privatecopy 검증 뒤 metadata exit41/download exit42, 취소는 cookie read 완료 후 cancel-ready CAP fsync→8초 observer bounded poll로 실행 중 요청한다. readiness 미관측/예상과 다른 오류/TerminalUnconfirmed는 통과시키지 않고 defer한다.

metadata 실패는 strict metadata Execution/Exited41→Failed/artifact0, download 실패는 download Execution/Exited42→Failed/artifact0, cancel은 download Execution/Cancelled→Cancelled/artifact0를 요구한다. 모든 반환 오류 로그는 SESSION_OUTPUT_OMITTED와 일치해야 한다. 실제 managed/cookie receipt수 2/1 또는3/2를 확인하며 원256 bytes를 owned root/receipt-evidence에 사본·SHA로 보존한 뒤 public cookie cleanup/managed finish·전체9files 제거/reuse를 확인한다. source/sentinel/tool hash·DB/WAL/SHM privacy는 공통 검사다. JSON은 scenario/passed/actual_execution과 기존 finish/source/cache/DB/omission/owner cleanup 인수 필드를 유지한다. metadata 실패에서 download 미실행을 확인하고 두 단계 path equality를 성공했다고 표시하지 않는다.

정확 실행 예: 기존 private windows-cli-host argv로 `fixture.exe --fixture-run fresh-metadata.json --scenario metadata-failure`를 전달한다. download-failure/cancel도 각각 fresh output으로 호출한다. root 소유 `scripts/windows_paired_download_faults_probe.py`가 build/privatehost/evidence를 담당한다. 실제 Cargo/Windows 실행은 아직 없으며 root integration-validation pending. staging `rustfmt --edition 2024 /tmp/dlpad-paired-faults-ready.rs` 실제 exit0. 기존 성공 인수 유지, parent crash/실제 계정·영상·네트워크 미시험. Git invalid 기존 .git 보존, Cargo/sharedAPI/다른 source 수정 없음.

주 담당 실제 Windows 인수: `windows-paired-public-download-faults-first`에서 metadata exit41/Failed, download exit42/Failed, actual reader 취소/Cancelled 세 경로가 통과했다. `root-review.json`에서 원 DB download_jobs 상태와 artifact0, 원 cookie/tool SHA, sentinel, receipt 사본 SHA/256바이트/managed magic, cookie cache 소유 기록 부재와 managed 9파일/pending 부재를 독립 확인했다. 첫 root SQL의 jobs 테이블명 오류는 실제 download_jobs schema로 정정했다. 최초 workspace strict Clippy는 fixture의 collapsible_if 경고로 실패하여 조건문을 동등한 let-chain으로 수정했다. 원 실패 증거를 보존하고 최신 전체 검증은 별도 재실행 중이다. 부모 crash/자동 startup 복구는 아직 미완료다.

최종 재검증: `windows-paired-faults-workspace-clippy-retry` Windows GNU CPU fmt/clippy/test/release 모두 PASS, 각 final source SHA를 현재 소스와 대조했다. 수정 후 `windows-paired-public-download-faults-clippy-retry` 실제 세 native case도 재통과하고 root-review.json의 원 DB/receipt SHA/원본·sentinel/소유 기록 부재 대조가 통과했다. 문서 검사 604개 PASS. 부모 crash/자동 회수는 후속이며 전체 P0 완료가 아니다.
