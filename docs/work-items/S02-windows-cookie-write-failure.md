# S02: Windows 쿠키 쓰기·동기화 실패 정리 회귀

- 상태: done
- 분류: test
- 우선순위: P0
- 요구사항: R04, R08, R11
- 담당/수정 범위: stress_recovery — cookies/windows.rs 및 본 문서만
- 선행 작업: Windows OwnedCopy 쓰기 전 RAII 구성; root native GNU 시험 실행

## 사용자 결과

private 복사본에 일부 또는 전체 bytes를 쓴 뒤 I/O 실패가 발생해도 확인된 소유 leaf만 정리하고 원본·외부 sentinel을 보존한다. reader 종료 미확인으로 보류한 copy는 계속 보존한다.

## 계약과 범위

제품 전역 env/failhook 없이 private owned writer closure helper로 실제 File write와 오류 주입을 분리한다. 공개 API/Cargo/schema/process 및 기존 backend 문서는 변경하지 않는다. unit 시험은 Windows cfg에만 존재하며 실제 OS temp files/privateACL을 사용한다. 주입한 sync 오류는 실제 물리 disk 장애 재현이 아니라 sync 단계 실패 반환 인수로 구분한다.

## 인수 조건

실제 partial write 뒤 WriteZero와 full write 뒤 injected sync I/O 오류 두 경로에서 fixed StorageUnavailable/owned owner만 제거/source·cache·foreign sentinel 보존을 검증한다. deferred OwnedCopy는 writer 호출 거절 후에도 copy/source가 남는다. root가 WindowsGNU actual unit과 기존회귀를 실행하며 작성한 시험을 PASS로 보고하지 않는다.

## 검증과 handoff

소유 파일 rustfmt와 문서 검사 후 READY SHA로 인계한다. Cargo는 실행하지 않는다. 실제 Windows native 결과·입력 SHA는 root 실행 뒤 기록한다.

## 구현 동결

private write_owned_copy(OwnedCopy,FnOnce(&mut File)->io::Result<()>)가 쓰기·sync 전체 동안 RAII를 유지한다. production closure는 기존 write_all→sync_all 그대로다. Windows unit3개는 실제 privateACL 임시 copy에 partial/full bytes를 써 조회한 뒤 WriteZero/동기화 단계 오류를 주입하고 고정 StorageUnavailable·정확 leaf 제거·원본과cache내foreignsentinel 보존을 확인한다. 세 번째는 이미 deferred된 copy를 helper가 다시 열거나 writer를 실행하지 않고 보존하는 회귀다. 물리disk syncfailure·실제 tool·OScrash 인수로 해석하지 않는다. root actual nativeGNU 실행 전까지 신규시험 PASS를 주장하지 않는다.

## root 통합 검증

Windows GNU CPU workspace test와 all-targets Clippy exit0. root가 before/after/current 입력 SHA 및 신규 실패 시험 PASS를 독립 대조했다. 원시 증거는 `docs/validation/evidence/startup-checkpoint-windows-retry/`이다. 최초 root test의 DuplicateHandle namespace 컴파일 실패는 별도 first evidence에 보존하고 수정했다. 실제 GUI·MSVC·macOS 검증은 포함하지 않는다.
