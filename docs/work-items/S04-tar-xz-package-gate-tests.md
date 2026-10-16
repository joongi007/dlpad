# S04: TAR.XZ package inactive install acceptance

- 상태: in-progress
- 분류: validation
- 우선순위: P0
- 요구사항: R05, R08, R13
- 담당/수정 범위: diagnostic_cache; crates/engine/tests/tar_xz_package.rs, 이 문서만
- 선행 작업: root v2 layout/installer dispatch/export

## 사용자 결과

합성 TAR.XZ가 inactive 설치→실제 inventory→명시 선택으로만 active가 되며 active 제거가 거절되고 다른 inactive 전체 제거가 원 source/외부 sentinel을 보존하는지 확인한다.

## 계약과 범위

기존 128byte fixture를 상수 재사용; 실행 가능한 실제 도구나 shell/네트워크/GUI 없음. ABC payload 버전을 검사했다고 주장하지 않는다. tempfile root 아래만 생성/제거한다. 원 archive SHA와 component SHA를 분리한다. Rust source/Cargo/schema 변경 없음.

## 인수 조건

- inactive/inventory/select/active guard/inactive remove 실제 ToolManager 경로.
- v1 missing layout decode None 및 serialize omitted; v1Some/v2None/invalidmapping 거절.
- v2 selected 합계384MiB 상한 및 v1 256MiB 상한 유지.
- archive wronghash, pinned truncatedXZ, archive hash는 맞지만 component hash가 다른 실패가 prior active/source/sentinel을 보존.

## 검증과 handoff

전용 파일 작성 후 ownfmt; root scoped integration test 대기. 실제 FFmpeg 공급/license/3OS/version 실행 증거로 확대하지 않는다. root가 기존 실제 API install_verified_archive 사용을 확정했다. old v1 byte JSON layout 생략, mapping unknown field 거절, manager 재개방 후 실제 inventory/active 보존 검사도 포함했다.
