# S04: Windows 관리 Deno 실행 환경 encoder 초안

- 상태: done (staging 구현 완료; 실제 통합 검증 대기)
- 분류: feature
- 우선순위: P0
- 요구사항: R04, R05, R08, R13
- 담당: metrics_recovery
- 수정 파일: /tmp/dlpad-runtime-environment.rs, 이 문서만
- 선행 작업: root의 ProcessSpec/config/caller 정책 연결 및 별도 원 cache ownership 검증

## 계약과 범위

프로젝트 Rust 게시/Cargo/공유 schema 변경 없이 순수 encoder를 staging 구현한다. WindowsManagedDenoEnvironment의 policy1/절대 cache 경로만 받아 ambient의 DENO_DIR/DENO_DISABLE_NODE_SHIM 모든 case-insensitive 중복을 제거하고 고정 두 값을 각 한 번 삽입한다. arbitrary caller 환경 override map을 제공하지 않는다. None은 기존 환경 전달을 바꾸지 않는다. Linux 제품 경로는 변경하지 않는다.

native cache 소유 proof, CreateProcessW 실제 환경 전달, EJS 및 yt-dlp 자식 전달은 root 후속이며 pure module을 제품 실행 완료로 표시하지 않는다. Debug/Error는 고정 정제 문자열만 가진다.

## 인수 조건

환경 변수4096개·UTF-16 block32767units·key256units 상한, cache32766units/절대 Windows drive·UNC와 parent/NUL/control/잘못된 UTF-16 거절, ASCII key 검증, 나머지 key 대소문자 중복 거절, double-NUL 및 고정두key 정확1회를 검증한다. 실제 Windows ProgramFiles(x86) 등 유효 printable ASCII key는 허용하고 '='·공백·control·nonASCII key를 거절한다. 환경 value의 잘못된 surrogate/NUL은 보존하지 않는다. 제거 대상 reserved value도 validation을 수행해 malformed ambient를 조용히 통과시키지 않는다.

## 검증과 인계

실제 runner는 root 통합 후 실행한다. 지금은 /tmp 구현과 정적 검토이며 Cargo/native 실행은 없다.

## 실제 구현과 확정 API

`/tmp/dlpad-runtime-environment.rs`에 실제 구현을 작성했고 개별 rustfmt exit0이다. SHA256 `e46db21577cf38c4fb0204c00787e3f33349748e8b8aac98faad1bcfc9c887ff`. 프로젝트 Rust/Cargo 파일은 게시·변경하지 않았다. root가 선언/통합 후 실제 runner를 실행한다.

공개 type `WindowsManagedDenoEnvironment`는 clone 가능한 struct이며 cache_path:PathBuf/policy_version:u8 필드는 private다. Windows에서 crate-private new(PathBuf)는 policy1로 고정하고 pure validate 뒤 반환한다. encode<I: IntoIterator<Item=(OsString,OsString)>>(&self, ambient)->Result<Vec<u16>,EnvironmentError>, validate(), cache_path()->&Path, policy_version()->u8를 제공한다. 고정 EnvironmentError는 InvalidPolicy/InvalidCachePath/InvalidKey/InvalidValue/DuplicateKey/BoundExceeded이며 Display/Debug에 실제 value/path를 넣지 않는다. root는 필요하면 io::ErrorKind::InvalidInput으로 매핑한다.

플랫폼 독립 `validate_cache_units(&[u16],version)->Result<(),EnvironmentError>`는 helper의 Vec<u16> wire 검증에 재사용할 수 있다. cache path는 Unicode drive 절대 경로와 일반 UNC server/share를 허용하고 verbatim drive prefix를 인식한다. extended UNC/device/GLOBALROOT는 명시 미지원 오류이며 native owner 권한이 아니다. ADS·parent·control·잘못된 UTF-16·trailing dot/space를 거절한다. Root의 ACL/FileId/cache proof는 별도다.

ambient는 key/value 각 bounded native encode_wide로 읽는다. 두 reserved key는 malformed value 검증 뒤 제거하고 나머지 case-insensitive 중복은 거절한다. block은 case-insensitive ASCII 순서로 정렬하고 실제 길이32767이하/최종 double-NUL을 만든다. 입력4096개와 출력4096개 상한을 모두 적용한다. 값과 실행경로는 Debug에 노출하지 않는다.

순수 회귀6개와 cfgWindows native OsString 회귀1개를 staging에 작성했다. reserved 중복/Unicode literal, PATH 중복/invalid key, NUL/surrogate와 reserved malformed, path/domain, 정확32767 경계/변수overflow, redacted Debug, Windows OsString invalidsurrogate를 검사한다. 아직 실제 테스트를 실행하지 않았으므로 PASS라고 기록하지 않는다. None 기존 상속 경로와 Linux 제품 코드는 바꾸지 않았다. 실제 CreateProcess Some 정책 전달과 EJS/yt-dlp 자식 전달은 root 후속 인수다.


Root 통합 후 inherited Windows 선두 '=' pseudoentry를 보존하는 회귀와 실제 원소유 자식 환경 전달을 추가했다. 기존 embedded '='·NUL/UTF16/중복/상한 오류 거부는 유지했다. 초기 key '=' 전면거부 계약은 실제 Windows 호환성 근거로 수정했으며 원 red loop는 보존했다. 최신 Windows fmt/clippy/test/release 모두 exit0: [근거](../validation/evidence/windows-runtime-environment-pseudo-fixed/root-review.json). private policy와 typed helper wire는 전달 문법이며 native cache 소유 authority나 전체 다운로드 성공을 뜻하지 않는다.
