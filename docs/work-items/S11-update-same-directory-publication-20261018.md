# S11: 동일 디렉터리 원 HANDLE publication

- 상태: in-progress
- 분류: bug
- 우선순위: P0
- 요구사항: R10, R13
- 담당/수정 범위: update_transport — `crates/engine/src/created_relative_file.rs`, 이 문서
- 선행 작업: S11-update-hosted-fixture-diagnostics-20261018, hosted stage publish 원 오류32

## 사용자 결과

앱 업데이트 패키지와 동일 primitive를 쓰는 journal 파일을 원 디렉터리 안에서 게시한다. 기존 사용자 파일을 덮어쓰거나 security/share 정책을 완화하지 않는다.

## 계약과 범위

Root가 API/schema/Cargo/버전/CI/공개를 담당하며 immutable v0.1.1 태그는 옮기지 않는다. 원 created source HANDLE와 strict simple leaf를 사용하는 native same-directory rename으로 한정한다. 다른 파일·디렉터리로 이동하는 API를 추가하지 않는다.

Microsoft [FILE_RENAME_INFORMATION 계약](https://learn.microsoft.com/en-us/windows-hardware/drivers/ddi/ntifs/ns-ntifs-_file_rename_information)은 동일 디렉터리 단순 이름 변경에서 RootDirectory=NULL을 지정한다. 기존 코드는 원 source와 동일 부모임에도 RootDirectory에 read-data/list/DELETE 관련 원 부모 HANDLE을 전달했다. hosted 실제 관측은 완결 partial ZIP 이후 publish의 OS32 sharing violation 및 publication/cleanup Unconfirmed다. target-directory 내부 재open과 현재 parent access/share의 상호작용은 강한 가설이나 정확 kernel share-check가 재현된 것으로 단정하지 않는다. FreshRoot 생성 HANDLE 자체는 share7이며 shareREAD-only라고 기술하지 않는다.

수정은 private rename helper의 불필요한 parent 인수를 제거하고 RootDirectory=NULL을 지정한다. 원 DELETE-capable source HANDLE, single-component validation(no slash/colon/traversal), ReplaceIfExists=false, 원 parent/file identity·path·ACL pre/post 검증, publication ambiguity 후 cleanup 권한 소실을 유지한다. CWD나 임의 전체 경로를 사용하지 않는다. retry·sleep·전체 sharing 확대·ACL 변경을 도입하지 않는다.

## 인수 조건

- 원 FreshRoot list/DELETE HANDLE와 별도 read-data parent HANDLE을 유지한 상태에서 동일 부모 publication·원 identity·bytes를 확인한다.
- 같은 상태의 기존 foreign target 충돌은 pre-call 거절하고 원 foreign bytes를 보존한다.
- 기존 race/postcheck/namespace 교체 실패 회귀를 유지한다.
- 실제 Windows native 및 hosted updater fixture를 재실행한다. 특정 OS의 통과를 미검증 OS에 확대하지 않는다.

## 검증과 handoff

수정 파일 rustfmt 완료. native target 함수 동작은 Root가 실제 MSVC regression과 hosted 실행으로 검증한다. before-failure는 실제 Windows Server2022 hosted 증거이며 로컬 Win11에서 before-failure를 새로 재현했다고 주장하지 않는다. 운영 key·계정·영상/사용자 Documents를 사용하지 않았고 원 단계 오류를 숨기지 않았다.
