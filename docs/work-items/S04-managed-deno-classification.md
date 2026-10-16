# S04: 선택된 관리 Deno 원 패키지 분류

- 상태: in-progress (Windows 분류 기반 통합·native 검증 완료; 생산 연결 대기)
- 분류: feature
- 우선순위: P0
- 요구사항: R04, R08, R11
- 담당/수정 범위: soak_recovery_audit; 본 문서와 신규 `/tmp/dlpad-managed-deno-classification.rs`만
- 선행 작업: ToolManager native root pin/full package inventory/checkpoint 및 runtime environment policy

## 사용자 결과

실제로 관리 설치된 Deno 선택 경로에만 관리 runtime 환경을 적용한다. 외부 Deno는 파일 변경이나 소유권 주장 없이 제외한다.

## 계약과 범위

원 root/directory/main executable native pin과 원 manifest/full package/hash/target를 검증한다. active pointer, 파일명 prefix, managed yt-dlp 선택 여부로 Deno 소유권을 추정하지 않는다. 루트 내부 불명·변조·reparse·hardlink는 fail-closed. root/shared module 연결과 cache 생성·native 실행은 통합 담당 범위이며 프로젝트 Rust/Cargo는 수정하지 않는다.

## 인수 조건

합성 관리 패키지의 active 아닌 명시 선택도 분류 성공. 외부 선택 None 및 byte 보존. 내부 malformed/unknown/main 아닌 경로/tool wrong/OS target mismatch/hash/extra file/hardlink/symlink 교체는 Err. 원 pin 변경과 full inventory 변경은 verify에서 거절. 원 checkpoint cancellation/timeout 실패를 그대로 전달한다.

## 검증과 handoff

임시 소스 작성·rustfmt 후 exact SHA/API를 root에 전달한다. Cargo/native 시험과 제품 게시 전까지 실제 인수 완료로 보고하지 않는다.

`/tmp/dlpad-managed-deno-classification.rs` SHA `0ec0df86f0340ca71f67c0484fbfbe6aced7238a6f1a4de55edc51ae1bba5701`를 동결했다. `rustfmt --edition 2024 /tmp/dlpad-managed-deno-classification.rs` exit0. 프로젝트 Rust/Cargo/설치 상태는 변경하지 않았다.

API는 `ToolManager::classify_managed_deno(selected, cancel) -> Result<Option<VerifiedManagedDeno>, ToolError>`이며 method/type 모두 crate-private이다. verified type은 `executable()`, `root_path()`, `verify(manager, cancel)`를 제공한다. root와 원 owner marker, package directory 및 모든 component/metadata 파일 pin을 유지하고 기존 full inventory/hash verification을 원 10초 absolute budget/cancel checkpoint 안에서 재사용한다. metadata JSON 비교와 native identity를 함께 검사하며 활성 pointer를 조회하거나 변경하지 않는다.

독립 재검토에서 Windows 기존 full inventory가 link count를 확인하지 않는 점과 plain/verbatim·case 별칭의 내부 미존재 경로가 None이 될 수 있는 점을 발견했다. 임시 수정본에 Windows `FILE_STANDARD_INFO` link count1/삭제예약 아님 및 Unix nlink1 검사(owner와 모든 파일의 전후), bounded512 ancestor REPARSE_POINT 거절, 미존재 경로의 deepest-existing-ancestor canonical 분류를 추가했다. 기타 permission/error는 보존하고 외부임이 확인된 경우만 None이다. Windows 전용 회귀 두 개는 canonical/plain/lower 내부 미존재 경로와 main/owner hardlink 증가·원 bytes 보존을 실제 API로 검증하도록 작성했다. 새 소스 역시 rustfmt만 실행했으며 실제 Windows 회귀 결과는 아직 없다. 최종 SHA는 root handoff 메시지와 임시 파일을 기준으로 한다.

준비된 회귀 소스 4개는 비활성 명시 경로/외부 None·byte 보존, main 아닌 notice/내부 불명, 6 변조·불명 모드와 취소, 전체 기록이 일치하지만 플랫폼이 다른 패키지, Unix 원 identity 교체·외부 symlink 내부 alias·hardlink 실패를 다룬다. 회귀는 아직 실행하지 않았다. root의 module/reexport 연결과 pure/native 실행 및 runtime policy 적용 인수가 남았다. 이 분류는 설치 bytes/소유 기록의 증명이며 현재 실제 Deno version 실행 증명을 새로 제공하거나 cache 소유권을 승인하지 않는다.

## 주 담당 통합 착수

root가 인계 SHA를 대조하여 crates/engine/src/tools/managed_deno_classification.rs, crates/engine/src/tools.rs에 게시했다. 공유 선언은 root만 수정했다. 생산 환경 활성화 전 기반 native 회귀를 실행하며 임시 dead-code 허용은 아직 미연결인 production factory/handoff에 한정한다. Cargo 의존성·lockfile·DB schema 변경은 없다. Windows 전체 검증 결과는 후속 기록하며 현재 컴파일/실행 인수 완료를 주장하지 않는다.

## Windows 통합 경계 보강과 실제 회귀

root가 수정 staging SHA `d6cc93864390ef98410fb57db23a4303c8724c2c20ca5e21830786a62e327d9b`를 대조하고 게시했다. 현재 Windows binding의 FILE_STANDARD_INFO bool 형식은 root가 조정했다. 독립 재검토에서 발견한 Windows hardlink 허용과 verbatim/plain/case 내부 미존재 경로 외부 오분류를 수정했다. 모든 원 owner/component/record에는 native single-link·DeletePending 검사를 반복하고, Windows ancestor reparse와 bounded deepest-existing-ancestor 판별을 추가했다.

[Windows 전체 시험 로그](../validation/evidence/windows-deno-owned-classification-cache-corrected/test.log)에서 분류 회귀 5개를 실제 실행해 통과했다. 비활성 관리 Deno 선택, 외부 bytes 보존, 전체 변조/취소, 일치하는 잘못된 OS, 원 main/owner hardlink 보존 거절, canonical/plain/lowercase 내부 미존재 경로를 확인했다. engine library는 292 passed/0 failed/2 ignored다. Unix 전용 원 identity/alias 회귀는 이번 Windows 실행에 포함되지 않으며 Linux/macOS 인수는 후속이다. 생산 cache factory와 metadata/download 정책 활성화는 아직 연결하지 않았다.

## 전체 통합 결과

[수정 후 Windows 전체 검증](../validation/evidence/windows-deno-owned-classification-cache-corrected/result.json)에서 `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets --locked --features cpu -- -D warnings`, `cargo test --workspace --locked --features cpu`, `cargo build --workspace --release --locked --features cpu` 모두 실제 exit0이다. [주 담당 독립 대조](../validation/evidence/windows-deno-owned-classification-cache-corrected/root-review.json)에서 현재 전체 입력 SHA와 전후 snapshot 및 현재 CPU release artifact 일치를 확인했다. 이 근거는 Windows GNU이며 GUI·MSVC·다른 OS·생산 helper/cache 적용 완료 근거로 확대하지 않는다.
