# S07: 명시한 prepared 폴더의 읽기 전용 재검증

- 상태: done
- 분류: feature
- 우선순위: P0
- 요구사항: R11
- 담당/수정 범위: root — app_install.rs 공유 interface·문서·통합·native 검증; soak_recovery_audit — app_install/windows_caps.rs 읽기 전용 cap; diagnostic_cache — 새 app_install/reopen.rs coordinator; metrics_recovery — 새 native fixture 초안 및 독립 검토
- 선행 작업: prepare transaction scoped DONE, source550 실제 copied app 두 private 시작 PASS

## 사용자 결과

명시한 이전 prepared 폴더를 별도 프로세스에서 다시 검사한다. 부분 복사·변조·외부 파일·busy writer를 거절하며 원 폴더와 사용자의 data/media를 보존한다.

## 계약과 범위

inspect_prepared(path, cancel, deadline) -> io::Result<PreparedInspection>. readonly 원 root/file HANDLE·currentSID ACL·no reparse ancestor·native identity·정확한 bounded ready record/namespace·전체 SHA/PE를 검사한다. PreparedInspection은 읽기 전용 관측이며 삭제/adoption/trust/자동 recovery 권한이 아니다. 공개 record()/path()/verify_current(cancel,deadline), executable은 원 읽기 pin File 참조로만 노출한다. schema/Cargo 변경 없음. root가 공유 API와 내부 cap 계약을 먼저 확정한다. updater 제외.

## 인수 조건

prepare 자연 종료 뒤 fresh inspection PASS; cancellation/deadline, marker 누락/oversize/unknown field/schema/domain/trust claim, 원 root/file ID·SHA 불일치, extra namespace, busy writer와 link/reparse를 거절한다. sentinel/모든 원 bytes 보존. 전체 native 검증과 실제 private launch는 root가 직렬화한다.

## 검증과 handoff

미구현 상태를 완료로 보고하지 않는다. 기존 prepared marker는 소유권 증명이 아니다. ancestor namespace 검사에는 pre/post 관측 한계가 있으므로 namespace lock으로 표현하지 않는다.

최초 통합 source552 Windows fmt/Clippy/workspace test/release 모두 PASS, 실제 app_install 28 tests PASS(한 test의 출력에 다른 테스트 prefix가 섞여 rawlog에서 직접 확인). CPU aefc4c27a43efe02bf6090ffa86851dfdb564ad97ef185b66015abd589a31537, 7,917,568B. 별도 Windows fixture에서 이전 source550 copied exe f1a9…를 새 프로세스로 inspect하여 원 ID/marker719/정확 namespace/모든 bytes 보존과 inspect/current 각각 취소·기한 실패 PASS. 현재 release 앱 자체 실행/설치 lifecycle 증명은 아니다.

독립 소스 검토의 실패범위 누락에 따라 marker schema/domain/trust/rootID/entryID 조작·foreign namespace 전체bytes보존·caller executable cursor 보존 3개 회귀를 추가했다. 기존 wrong_identity 이름은 실제 SHA 변조에 맞춰 wrong_sha로 고쳤다. 이 후속 변경 전체 검증은 진행 중이다.

최종 추가 실패 회귀 source552 전체4검사 PASS, 실제 app_install 31 tests PASS. CPU f274db5dcdc192b51343be3f2a23dbeeef57192a472b3bdda8a70455c2426836/7,917,568B, 이전 CPU 대비 0B. 최종 별도 native reopen fixture도 PASS, 읽기 전용 원 copied 이전 CPU f1a9…/marker/namespace/bytes 보존 및 네 typed 실패를 root가 재확인했다. evidence windows-app-prepared-reopen-failures-workspace-20261006-continue와 windows-app-prepared-reopen-failures-real-copy-20261006-continue. 최종 scope는 inspection이며 실제 실행 연결/자동 recovery/정확 제거/서명/installer 등록은 후속이다.
