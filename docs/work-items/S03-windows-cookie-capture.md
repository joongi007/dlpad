# S03: Windows authenticated capture backend

- 상태: in-progress
- 분류: feature
- 우선순위: P0
- 요구사항: R02, R08
- 담당/수정 범위: soak_recovery_audit — crates/engine/src/process/windows_cookie_capture.rs, 이 문서
- 선행 작업: root registration/process wrapper, diagnostic_cache WindowsGatedCookieReader, metrics helper native ownership

## 계약과 인수

실제 helper gated metadata/download, 원 pipe의 bounded capture, Prepared identity/durable binding/unlock/Ack/Begin, Terminal code/helper status/EOF와 original receipt 확인을 구현한다. cancel/timeout/outputlimit은 새로운 cleanup deadline으로 helper를 정지하고 불확정 reader는 pending으로 보존한다. 공유 source/Cargo/다른 담당 파일을 수정하지 않는다. malformed/partial/EOF/backpressure와 native 실패 경로는 root 통합 검증이다.

## Root 실제 연결과 검증

root가 process.rs Windows cookie 분기를 dedicated capture로 연결하고 download.rs의 Linux-only gate 분기를 Windows까지 확장했다(legacy begin_reader와 pending CAS 이중실행 방지). first native Clippy unused export/receipt anchor 미사용/testmodule배치로exit101, 실패 보존. root가 실제원receiptanchor대조와testmodule배치를수정한 second engine all-target Clippy exit0/sourceunchanged/inputdesktopunchanged. Windows originalExitStatus code bitpattern을 code() asu32로대조한다. 다운로드 adapter cfg변경은 이second검증 이후이므로 추가검증 필요. actualauthenticatedproduct 다운로드4case/전체workspace/GUI 인수는 아직남는다.

Windows adaptersecond engineClippy/appCPUClippy/buildguard/engine tests 모두exit0, engine156PASS/1ignored. 이후새nativeintegration4case thirdPASS를별도검증했다. 전체workspace/최신GUI는아직미검증이다.
