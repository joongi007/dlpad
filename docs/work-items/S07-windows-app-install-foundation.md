# S07: Windows 앱 설치의 검증된 로컬 payload와 소유 디렉터리 기반

- 상태: done
- 분류: feature
- 우선순위: P0
- 요구사항: R11
- 담당/수정 범위: root — crates/engine/src/app_install.rs, crates/engine/src/lib.rs, crates/engine/examples/windows_app_install_foundation_fixture.rs, 본 문서·공유 계약·통합 검증; diagnostic_cache — crates/engine/src/app_install/manifest.rs 및 app_install/payload.rs; soak_recovery_audit — crates/engine/src/app_install/windows_caps.rs; metrics_recovery — 독립 검토 /tmp만
- 선행 작업: Windows 개발 후보 ZIP·폰트 결합 검증; 진행 중 GUI 종료 후 Rust 소스 변경

## 사용자 결과

앱 설치 기반이 로컬 payload의 이름·크기·digest·대상을 엄격히 검사하고, 새 Windows 전용 설치 디렉터리의 원 identity와 private ACL을 확인한다. 기존 사용자 디렉터리를 이름만으로 소유하지 않는다. 이번 단위는 installer 전체 완료가 아니라 설치 coordinator의 실제 재사용 기반이다.

## 계약과 범위

공유 오류는 std::io::Error, 신규 의존성 없음. manifest는 serde deny_unknown_fields·bounded JSON reader·단일 ASCII 파일명 allowlist·중복 및 Windows 예약 이름 거절·총용량 상한·SHA256 형식·x86_64 Windows 대상만 허용한다. 개발 후보의 unsigned 상태를 신뢰된 공개 릴리스로 승격하지 않는다. Windows capability는 fresh create-only root, 원 File/native identity/final path/current SID/private ACL 보유 및 재검증이다. 기존 경로 adoption·재귀 삭제·권한 변경·global registry/PATH·updater는 없다. 인터페이스/Cargo/schema는 root가 관리한다.

## 인수 조건

manifest 파싱은 길이 상한+1에서 거절하며 traversal/예약명/대소문자 중복/잘못된 digest/overflow/unknown field를 실패 시험한다. Windows root는 기존 디렉터리·reparse 조상·identity 교체·ACL 차이를 거절하고 원 디렉터리만 검증한다. 원 파일 handle의 read/copy/hash 계약을 문서화하고 share-delete의 한계를 숨기지 않는다. root가 실제 Windows fmt/Clippy/test/release와 native capability 시험을 통합한다. publish/restart/uninstall/clean-machine은 후속 coordinator 단위이며 이 단위의 통과로 설치 인수를 완료 처리하지 않는다.

## 검증과 handoff

공유 계약 확정 후 독립 파일별 구현을 배정한다. Git 저장소가 정상 인식되지 않으며 기존 .git을 보존한다. 사용자 data/media는 합성 TEMP만 시험한다.

최초 Windows 전체 검사: fmt PASS, Clippy unnecessary_mut_passed 경고로 FAIL이며 test/release는 실행되지 않았다. 원 evidence `windows-app-install-foundation-workspace-20261006-continue`를 보존한다. 독립 capability 검토에서 CreateDirectoryW와 후속 open 사이 leaf 교체를 최초 identity로 채택할 가능성을 발견했다. 채택 전 실제 원 parent HANDLE에 상대적인 NtCreateFile(FILE_CREATE)로 원 생성 HANDLE을 함께 얻도록 수정한다. 기존 windows-sys0.61.2의 Wdk_Foundation/Wdk_Storage_FileSystem 선언 feature만 root가 추가한다. 신규 crate/download/runtime daemon은 없고 ntdll 기존 OS 호출을 사용한다. 최종 binary 크기 비용은 새 build 후 측정하며 Cargo.lock 변화 여부를 확인한다. [OS 계약](https://learn.microsoft.com/en-us/windows/win32/api/winternl/nf-winternl-ntcreatefile). 수정 후 fresh 전체 검증이 필요하다.

후속 병렬 범위: payload.rs는 descriptor와 원 borrowed File의 정확 세트·native identity 전후·길이·64KiB streaming SHA·bounded PE64 구조·취소/기한을 검증한 관측 자료만 반환한다. immutable provenance·실행·삭제 권한은 주지 않으며 coordinator가 deny-write source pin과 재검증을 별도로 유지해야 한다. shared cursor 대신 offset read를 사용한다. 해당 실패 경로 시험을 실제 tempfile에서 수행하며 root가 module 등록과 통합을 맡는다.

2차 소스 검토: 원 parent-relative NtCreateFile(FILE_CREATE) 생성 HANDLE과 lexical leaf 거절·원 HANDLE 교체 회귀를 통합했다. Payload의 Windows seek_read는 명시 offset을 사용하지만 caller cursor도 바꾸므로 원 HANDLE 기반 ReOpenFile로 독립 커서를 얻는 수정을 진행한다. File::try_clone을 독립 Windows 커서로 오인하지 않는다. [Rust OS 계약](https://doc.rust-lang.org/std/os/windows/fs/trait.FileExt.html). 수정 후 원 caller cursor 보존 시험을 실제 Windows에서 확인한다.

2차 전체 검사: fmt PASS, Clippy compile에서 OBJECT_ATTRIBUTES의 typed SecurityDescriptor와 SECURITY_ATTRIBUTES의 void pointer 타입 차이로 FAIL(E0308)했다. 원 backing descriptor 수명을 유지하고 .cast()로 정확 binding pointer 타입을 맞췄다. 원 실패 `windows-app-install-foundation-fixed-workspace-20261006-continue`를 보존하며 test/release는 아직 실행되지 않았다.

Native fixture 추가: 명시 flag/절대 CPU 경로/고정SHA를 받고 원 source FILE_SHARE_READ와 no-reparse handle에서 실제 PE/hash/nativeID 및 cursor17 보존을 확인한다. 별도 합성 notice와 sentinel·새 한글 private root의 cap.verify를 연결한다. createNew TEMP를 유지하고 source 복사/실행/설치·제거 완료를 주장하지 않는다. 기존 prototype를 root가 코드 검토·통합했으며 실제 실행 전이다.

최종 scoped 인수: source548의 실제 Windows fmt/Clippy/test/release 모두 PASS. manifest6·root5·payload4 신규15개는 actual ok이며 ignored가 아니다. 현재CPU 4f7a5fdd8e92fa7df266c2463b1c053ec49076fd0ad45633348ca51acc82fa05/7,917,568B, 이전 대비0B·Cargo.lock 동일을 확인했다. [전체 root 검토](../validation/evidence/windows-app-install-foundation-typed-workspace-20261006-continue/root-review.json). 실제CPU/notice의 PE·streamSHA·원 nativeID 및 cursor17→17·새 한글root/privateACL·sentinel 보존과 자연Job0도 통과했다. [실제 payload 검토](../validation/evidence/windows-app-install-foundation-real-payload-20261006-continue/root-review.json). root ACL tamper·reparse 조상 별도 새 API 실패 fixture, crash/reboot/실제 install/restart/remove/clean-machine은 미검증이다. 이 기반 완료는 installer/P0 전체 완료가 아니다. 초기 두 Clippy 실패도 보존한다.
