# S04: Deno managed version-discovery reader

- 상태: in-progress
- 분류: feature
- 우선순위: P0
- 요구사항: R03, R05, R08, R13
- 담당/수정 범위: diagnostic_cache; crates/engine/src/deno_runtime.rs, 이 신규 문서만
- 선행 작업: root ManagedExecution.run_tool/ManagedReaderStage/export; download caller root 담당

## 사용자 결과

Windows 관리실행scope에서Deno --version도VersionProbe reader로guardian/원receipt를거쳐실행한다. 별도runtime 원file/hash/namespacepin검증을생략하지않는다.

## 계약과 범위

기존 inspect(path,cancel) wrapper 유지→inspect_with_managed(path,cancel,None). 새helper는managed Option<&ManagedExecution>;Windows Some일때run_tool(spec,cancel,true,None,VersionProbe),None/nonWindows는기존run_metadata. 원10초budget/4096strictstdout/maxcapture/argv/원posthash/cancelkind/terminalunconfirmedio유지. options snapshot에executioncapability 저장하지않는다. 외부source 삭제/변경없음. rootdownload.rs caller는같은ownedmanagedscope를전달한다.

## 인수 조건

기존Linux synthetic 정상/malformed/nonzero/mutation/cancel 회귀와commonClippy root통합검증. Windows native syntheticDeno correctargv/recordVersionProbe exactreceipt/managedsnapshot재개, wrongversion/nonzero/cancel/mutation 보존. helper Job/terminal 미확정은원ioerror를그대로전달하여SessionCleanupDeferred gate 보존. 아직미실행 OS완료claim없음.

## 검증과 handoff

최소helper변경/ownfmt후sourcefreeze; Cargo/build/tests는root직렬통합실행. 다른Rust/Cargo/shareddocs무수정.

주 담당 통합 검증: windows-managed-download-propagation-frozen-retry에서 실제 Windows GNU CPU 전체 fmt·Clippy(-D warnings)·workspace test·release 통과 및 현재 소스 SHA 일치를 확인했다. windows-managed-public-download-first는 공개 Repository에서 VersionProbe→Metadata→Download, Downloaded DB 저장, 실제 합성 artifact, 원 journal 정리·다음 실행 재사용을 통과했다. 주 담당이 DB의 job_json, 실제 파일 bytes, 3단계 marker, 원 도구 SHA/sentinel 및 9개 journal 파일 부재를 별도 읽어 대조했다. 초기 독립 SQL 조회는 state 컬럼 대신 실제 job_json 구조를 사용하도록 바로잡았다. Deno/FFmpeg 실제 지정 경로와 mixed 인증 paired runtime 및 startup 자동 회수는 아직 미검증/미완료다.
