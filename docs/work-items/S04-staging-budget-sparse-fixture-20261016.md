# R05/R08: Windows staging budget 경계 시험의 sparse fixture

- 상태: prepared / Windows 실제 미실행
- 분류: test-fixture fix
- 우선순위: P0 검증 선행
- 요구사항: R05(관리 도구), R08(안전 복구), R10(재현 검증), R11(Windows 빌드)
- 담당/수정 범위: p0_audit는 이 scratch의 staging_recovery.rs 후보만. Root는 S/Live 통합·Cargo/native 검증 담당. 제품 source/Live/S/Cargo/lock/원증거는 변경0.
- 선행 작업: E/windows-msvc-ui-feedback-v30-frame-tail-feature-v1-20261015의 원 Windows StorageFull112 실패·기본3 복구 결과 보존. Root 소유 cache/디스크 정리는 별도 작업이다.

## 사용자 결과

Windows 테스트가 논리 크기512MiB 경계를 검증하기 위해 실제 디스크512MiB를 먼저 할당하지 않도록 합성 fixture를 sparse 파일로 만든다. 제품 staging 총량 제한512MiB, foreign file 보존, 실제 logical size 검증은 그대로다. 이 변경 자체가 제품 디스크 부족 처리나 strict CPU 문제를 고친다는 의미는 아니다.

## 계약과 범위

aggregate_bytes_plus_new_reservation_never_exceed_budget 시험의 새 빈 fixture에만 cfg(windows) 블록을 넣는다. 기존 writable 원 file HANDLE로 FSCTL_SET_SPARSE를 호출하고 성공 및 원 file metadata sparse attribute를 assert한다. 실패는 last_os_error와 함께 시험 실패이며 skip/dense fallback/예산축소가 없다. 기존 Win32_System_IO·Storage_FileSystem feature를 사용한다. 잠긴 windows-sys0.61.2 generated source와 같은 control code를 test-only 상수로 두어 Win32_System_Ioctl 의존 feature를 추가하지 않는다. Linux 경로는 기존대로 set_len만 수행한다.

## 인수 조건

실제 Windows에서 sparse 설정을 성공해야 한다. 기존 MAX_STAGE_BYTES-10 logical 크기, reserve11 거절/reserve10 성공/원unknown-original 크기 보존, MAX_STAGE_BYTES+1 이후 recover 거절의 원 assertions가 모두 그대로 통과해야 한다. 실패시 명확 FAIL로 남긴다. 같은 fixture에서 두 set_len 모두 sparse 상태를 유지한다. 예산/실패경로/foreign preservation 정책을 변경하지 않는다.

## 검증과 handoff

scratch rustfmt와 원문+windows-only 블록 구조 대조만 수행한다. Cargo/Windows 실제 sparse·실제 할당량 측정은 Root 미실행이다. Root는 filtered engine test→관련 staging recovery 회귀→기존 feature4 재실행 순서를 조정한다. 원 StorageFull112/feature release미실행/기본3exact 복구 결과를 그대로 보존한다.

실제Windows S의staging_recovery필터2PASS/0FAIL(0.05초)을확인했다. 원512MiB논리경계·foreign보존assertions를그대로통과했고Sparse설정/원HANDLEattribute도성공했다. 실제디스크할당량은별도측정하지않았다. 새의존성/lock변경없고아직Live미채택이다.

Root가실제S검사·원소스674/현재12파일·lock불변을대조해source31에채택했다. source675의새Windows전체검사실행중이며현재사용자v30 ZIP과구별한다. 격리최종strictworkspace/all-targetsclippy는0이다.
