# S11: Hosted Windows 파일 공개·교체 실패 수정

- 상태: in-progress
- 분류: bug
- 우선순위: P0
- 요구사항: R08, R10, R11
- 담당/수정 범위: Root — Windows 파일 공개/쿠키 marker 회귀와 관련 문서·workflow·통합 검증·push. Cargo/schema 변경은 필요 시 Root 조정.
- 선행 작업: S11-hosted-ci-lint-125-20261017, 실제 run38949294761/ebed507

## 사용자 결과

Lint 통과 후 실패한 Windows 테스트를 실제 원인에 맞게 수정하고 원격 CI 전체를 다시 실행한다. 테스트를 제외하거나 성공으로 위장하지 않는다.

## 계약과 범위

원 artifact ZIP183051B/SHA256 53dce067252b9e6e8ef2f840677f56d245c40f7d42b5190997ab336270410f22는 로컬 ignored capture-ebed507에 보존했다. test worker101/naturalJobemptytrue/forcedfalse이며 engine554PASS/22FAIL/17ignored다. 원인 후보는 Server2022 rename 제약, 원 파일/부모 HANDLE 공유 모드 충돌, 테스트 간 간섭 순이다. 원 핸들 identity·외부자료 보존·create-only 공개·충돌 거절 정책을 유지하고 임시 합성 파일/격리 Windows host만 사용한다. 실제 계정·사용자 미디어·보이는 데스크톱 사용0이다.

관련 문서: README→PRODUCT→CONTEXT→R08/R11, ADR0002/0005, ARCHITECTURE/INTEGRATIONS. 먼저 Windows caps publish와 cookies marker의 최소 재현을 확인하고 원 호출 경계 회귀를 보존한다. 상세 수정 파일은 확인 결과로 갱신한다.

## 인수 조건

- 원 오류 NTSTATUS0xc0000043/Win32 error32를 최소 재현과 연결한다.
- 파일 공개·경쟁·교체·취소·외부파일 보존 회귀가 실제 Windows에서 통과한다.
- 필수 문서/Rust 검증과 새 GitHub 실행을 확인하고 실패/skip/미검증 범위를 구분한다.

## 검증과 handoff

진단 중. 원 hosted 로그는 확인했으며 로컬/수정 CI 성공은 아직 주장하지 않는다.

## 최소 비교와 수정 후보

기존 원격 실패는 설치 계열21개와 cookies marker 교체1개다. Windows11 build26200 private host의 합성 ctypes 최소 비교에서는 RootDirectory=원부모/NULL 두 NT rename과 DELETE capability 부모를 유지한 MoveFileEx가 모두 성공했다. 원 Job naturalzero·forcedfalse·데스크톱/소스불변을 확인했다. 따라서 local에서 Server2022 실패를 재현했다고 주장하지 않으며 원 hosted 테스트를 RED 기준으로 유지한다.

[Microsoft FILE_RENAME_INFORMATION](https://learn.microsoft.com/en-us/windows-hardware/drivers/ddi/ntifs/ns-ntifs-_file_rename_information)는 동일 폴더의 단순 이름 변경을 RootDirectory=NULL로 정의하고 별도 대상 부모 open의 공유 충돌을 설명한다. windows_caps.rs의 원 file HANDLE/DELETE capability·bounded leaf·replace=false를 유지하면서 RootDirectory만NULL로 바꾼다. parent path 재개방/fallback·share-delete 완화·POSIX bypass를 넣지 않는다. 기존 원 실제 publication·collision·foreign mutation·경쟁 테스트가 회귀 seam이다.

cookies/windows.rs는 제품 동작 변경 없이 marker 교체 시험의 주입 경로만 수정한다. 원 marker의 ReOpenFile DELETE capability와 NT 동일부모 rename으로 원 marker를 별도 이름에 보존한 뒤 동일 bytes의 새 marker를 만든다. std::fs::rename의 부모 경로 재개방 실패 때문에 identity 검증 전에 중단되지 않게 하며 실제 CleanupOwnershipChanged와 원 쿠키/외부 sentinel 보존을 요구한다. 생산 marker의 공유 권한·삭제 정책은 바꾸지 않는다. 신규 의존성/도구/schema/추가 runtime HANDLE0이다(시험용 HANDLE은 RAII로 닫음).

## 로컬 검증 진행 기록

source-only docs1118개·원 로컬 docs1213개/133개 upstream notice 검사가 통과했다. 첫 native Rust 검증은 fmt0 후 기존 GNU 환경 누출로 MSVC build-script link를 잘못 선택해101, 두 번째는 PATH의 GNU Cargo subcommand 영향으로 link.exe 누락101이었다. 두 원 로그와 자연Job정리/데스크톱 불변을 보존했으며 제품 테스트 실패나 성공으로 집계하지 않는다. 세 번째는 기존 검증 MSVC의 CARGO·PATH·RUSTC·linker·compiler 환경까지 맞춰 실행 중이다. Windows11 로컬과 Server2022 hosted 결과를 구분하며 원격 재실행으로 최종 판정한다.
