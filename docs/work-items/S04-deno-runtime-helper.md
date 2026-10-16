# S04: 외부 선택 Deno 원 실행 파일 검증 helper

- 상태: in-progress
- 분류: feature
- 우선순위: P0
- 요구사항: R05, R07, R11
- 담당/수정 범위: diagnostic_cache — 신규 crates/engine/src/deno_runtime.rs 및 이 문서
- 선행 작업: root tools strict pure parser/lib export/download worker 연결

## 사용자 결과

명시 Deno 경로를 원 ancestry/file native IDs·해시와 strict stable hosttarget version 출력으로 검증하고 stage 전 재검증할 수 있다. 외부 파일 수정/삭제0.

## 계약과 범위

inspect(path,&Arc<AtomicBool>)->io::Result<SelectedDeno{path,version,privatepins}>. verify(cancel)로 원native IDs/hash/ancestors 재확인. inspect 전체10초·hash256MiB/64KiB·argv --version만·log4096, strict stdout postcap4096. 기존run_metadata stdoutcapture8MiB이며 actual4096capture API는없어 root정책대로비용기록후oversize거절. longI/O rootworker만. Cargo/다른Rust/공통문서미수정.

## 인수 조건

Linux synthetic native version 정상/wrongversion/nonzero/selfmutation/cancel·원ID변경·unsafe경로 거절, Interrupted/TimedOut/InvalidInput/InvalidData 분류 유지. Windows read payload는 attributes-only helper가아닌 readtrue+OPEN_REPARSE_POINT. native ancestors pin/readattributes/reparse·type검증 유지.

## 검증과 handoff

source READY/frozen. inspect는 absolute/noParent/64components/32766pathbound/noSymlink 및 Windows reparse 속성을 검사한 뒤 canonical path와 모든 ancestor original directory handles·native IDs를 보유한다. Windows payload는 read(true)+OPEN_REPARSE_POINT이며 디렉터리만 FILE_READ_ATTRIBUTES/BACKUP_SEMANTICS를 쓴다. actual handle type/tag와 regular-under-last-parent를 검사한다. 원 file handle/현재 path handle의 IDs와 SHA를 실행 전후 비교하고 SelectedDeno::verify로 metadata/download 각 stage 직전 재검증한다. root pure verify_deno_output bool reexport와 lib/download 연결 대기다.

고정 지원 target은 Linuxx86_64 GNU, Windowsx86_64 MSVC runtime(GNU 앱 호환), macOS x86_64/aarch64다. 실제 OS 실행을 추가한 것은 아니다. exact stable3line 출력에서64byte이하3개numericsegment 버전을 추출한다. minimum EJS호환/실제사이트를 이 exactversion 관측으로 증명하지 않는다.

Linux Python shebang 합성 argv-only 실제 실행 시험3개를 작성했다: 정상버전+post nativeIDreplacement 거절, malformed/nonzero/실행중ownedsyntheticfile selfmutation/selectedsymlink 거절, ready marker확인후ownedcancel Interrupted 및sourcebytes보존. 테스트 source mutation은이번소유fixture만이며 실제 externalfile 수정권한은없다. Cargo/individualrustfmt는 root공유검증을 위해 수행하지 않았다. 실제실행결과 대기.

run_metadata는 stdout최대8MiB를 retain하고 반환후4096byte엄격거절한다. max_log4096은stdoutcapture4096와같지 않다. root가 기존선택API와같은비용정책을 승인했으며 새process API를 추가하지 않았다. hash는256MiB·64KiB fixedbuffer지만 repeatedhash/nativeversion 전체10초로timeout을유지한다. verify는stage별새10초budget이다. namespace 검증과 path로실행 사이 완전한 원자 방어를 주장하지 않는다. 동일사용자의교체/복원경합을nativeIDs/hash만으로완전히차단했다고표현하지 않는다.
