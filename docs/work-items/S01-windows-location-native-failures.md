# S01: Windows 저장 위치의 실제 경로·실패 인수

- 상태: implementation-ready (/tmp 실행 예제 준비; Windows 실제 실행 대기)
- 분류: test
- 우선순위: P0
- 요구사항: R02, R01, R11
- 담당/수정 범위: soak_recovery_audit; 본 문서, `/tmp/dlpad-windows-location-native-fixture.rs`만. 프로젝트 Rust 게시·빌드·native 실행은 root 담당.
- 선행 작업: location/library/metrics 공개 API, S01-storage-selection; root 전체 검증 session6462 종료 후 게시 승인

## 사용자 결과

한글·Unicode와 리디렉션을 닮은 합성 저장 위치에서 생성·재사용·선택 저장·복귀를 확인하고, 실패가 기존 선택과 원 파일을 훼손하지 않는지 Windows에서 검증한다.

## 계약과 범위

README→PRODUCT→CONTEXT→R02/R01/R11→ADR-0002/ARCHITECTURE 및 기존 저장 위치 작업을 읽었다. `location::prepare_with_default`는 시험 공급자를 주입하며 `library::default_library_root`는 실제 OS Documents 조회만 한다. directories/dirs-sys의 Windows 구현은 SHGetKnownFolderPath를 사용한다. 실제 Documents에는 prepare/create를 호출하지 않는다. 합성 OneDrive 디렉터리는 실제 리디렉션 프로필·클라우드 동기화 인수가 아니다. 기존 .git과 사용자 데이터·ACL·전역 설정을 건드리지 않는다.

## 인수 조건

fresh tempfile에서 한글 사용자명 형태/OneDrive 형태/비BMP Unicode 경로를 생성·재사용하고 literal 원 sentinel을 보존한다. 기본 조회 실패·file-as-directory·상대경로·data overlap 뒤 기존 선택이 Restore되고 실패 target이 생성되지 않아야 한다. 실제 KnownFolder 조회는 생성 없이 별도 결과를 기록한다. 대상 volume 실제 값과 missing-path 오류를 구분하며 오류를 0bytes 또는 Unknown 지원 성공으로 바꾸지 않는다.

## 읽기 감사 결과와 잔여

location 기존 Windows 공통 시험은 기본 생성·선택 저장·조회 실패·file/overlap 보존을 포함한다. 권한 거절 생성 시험은 Unix 전용이며 Windows 실제 ERROR_ACCESS_DENIED 생성 경로는 현재 빈틈이다. 주입 공급자의 PermissionDenied는 API 조회 실패 회귀이며 실제 Windows 폴더 ACL 거절 증거가 아니다. 새 예제에서 둘을 명확히 구분한다. 실제 KnownFolder가 OneDrive로 설정된 계정, Windows rootpath, UNC/분리 볼륨, native 권한 거절 생성은 후속 별도 fixture가 필요하다.

location의 symlink 검사는 최종 leaf만 검사하며 Windows junction/reparse 및 중간 alias 전반의 보안 경계를 증명하지 않는다. 현재 요구상 저장 위치는 사용자 선택 외부 위치를 허용하므로 이 관찰을 곧바로 결함으로 확정하거나 cookie/cache의 엄격한 소유 정책으로 변경하지 않는다.

## 검증과 handoff

Cargo/native/GUI는 실행하지 않았다. /tmp 예제 READY 후 root가 immutable native exe를 hidden CLI host에서 실행한다. 출력은 합성 경로 원값·OS 사용자 경로 대신 scoped boolean과 실제 free-byte 결과만 반환한다. 아직 실제 Windows 인수 완료가 아니다.

`/tmp/dlpad-windows-location-native-fixture.rs` SHA `495fa93055840ff1dd2b94dd9d27d3064cbae2d7f0ac4ee7415c2d3ba78c0eb8`를 준비했고 소유 사본에 `rustfmt --edition 2024` exit0을 확인했다. root 게시 제안은 engine example `windows_location_native_fixture`이며 argv 없이 실행해 schema1 JSON 한 줄을 반환한다. 실제 native root volume은 조회만 하고 drive-root에 저장 폴더를 만들지 않는다. Windows malformed UTF16 선택도 UnsupportedPath 및 기존 선택 보존을 요구한다. 실제 빌드·public API 연결·native 결과는 아직 미검증이다.

## 2026-10-04 root 통합 및 실제 실행

crates/engine/examples/windows_location_native_fixture.rs를 SHA검증 후 통합했다. Windows 고정 도구에서 cargo fmt --all -- --check와 cargo run -p dlpad-engine --example windows_location_native_fixture --locked를 실행해 모두 통과했다. actual JSON은 synthetic_unicode_redirect_create_reuse/saved_selection_preserved_after_failures/original_sentinels_preserved/malformed_utf16_rejected/missing_volume_error/windows_root_volume_lookup 모두 true이다. Windows 실제 Documents lookup는 읽기 전용으로 가능했고 실제 Documents 생성0. 실제 OneDrive profile/native ACL permission denial/unknown volume/root save는 false를 유지한다. windows-location-native-20261004-resume/root-review.json에 root 독립 결과를 기록했다. 전체workspace 회귀는 별도로 진행중이다.

Windows GNU CPU全workspace 후속 fmt/Clippy/test/release 네명령 PASS이며 root가 current source_final SHA일치와 각exit0 및 실제 release binary SHA를 확인했다. 근거 windows-location-integrated-workspace-20261004-resume/root-review.json. ignored시험은 실제실행완료로 확대하지 않는다.
