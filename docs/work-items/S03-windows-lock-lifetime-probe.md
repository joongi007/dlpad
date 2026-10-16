# S03: Windows 복제 핸들 잠금 수명 실험

- 상태: done
- 분류: prototype
- 우선순위: P0
- 요구사항: R08, R11
- 담당/수정 범위: metrics_recovery — 신규 scripts/windows_lock_lifetime_probe.rs, 이 문서 및 전용 새 evidence만
- 선행 작업: Windows journal 비차단 File lock; 독립 guardian 잠금 handoff 계약

## 사용자 결과

잠긴 원래 File의 복제 핸들을 실제 child에 상속하고 부모 원본을 닫은 뒤, 별도 open의 try_lock 결과로 lock 유지 여부를 측정한다. 결과가 유지되지 않으면 shared Unix OFD처럼 간주하지 않고 guardian 자신이 잠금을 획득하는 별도 gate 계약이 필요하다. 제품 startup 완료를 주장하지 않는다.

## 계약과 범위

standalone Rust GNU native probe이며 synthetic private temp만 사용한다. parent File::try_lock→try_clone/child stdio handle 전달→child actual native FileId 확인→부모 duplicate 및 original close→별도 open try_lock을 관측한다. child는 원래 stdin handle을 보유하고 bounded marker handshake로 종료한다. child 자체와 original process handle은 유한 wait/kill/reap이며 guessedPID signal은 없다. host는 기존 hidden CLI private desktop/owned Job를 사용한다. 사용자 화면·데이터·전원·보안·Cargo·제품 source/schema 변경은 없다.

## 인수 조건

원래 lock 보유 중 다른 open은 WouldBlock이어야 한다. child ready 및 original128bit FileId 일치를 확인한 뒤 original close 경계의 actual result를 기록한다. child 종료 후 새 open은 lock을 획득해야 한다. 유지/미유지 모두 실제 관측으로 보고하고 설계 영향은 별도로 기록한다. query 실패/identity불일치/timeout을 성공이나 unlocked0로 쓰지 않는다. source/tool hash와 raw 로그, 정확한 argv/exit/elapsed/fixture 정리를 기록한다.

## 검증과 handoff

실제 결과는 실행 후 기록한다. native 단발 fixture이며 장기 동시성·guardian crash·제품복구 또는 lock duplication의 모든 OS 의미를 입증하지 않는다.

## 실제 결과와 설계 영향

[전용 evidence](../validation/evidence/windows-lock-lifetime-20261004T010744Z/plan.json)에 정확한 compiler argv/tool hash/bytes를 저장했다. 기존 windows_current_workspace_probe의 hidden CLI host/run/configure를 재사용하여 standalone rustc GNU edition2024 -O compile exit0 후 native 3회 모두 exit0/private-desktop=true/input-desktop-unchanged/source_unchanged=true를 확인했다. 정확한 native argv와 before/after manifest는 native-parent.json, native-parent-2.json, native-parent-3.json에 있다.

첫 실행은 host가 추가argv를 요구하여 usage exit2로 fixture에 진입하지 않았다. native.log/result.json에 그대로 보존했다. 동일 artifact에 --parent 인수만 추가한 native-parent.log 및 반복2/3으로 실제 실행했다. 실패를 성공으로 대체하지 않는다.

세 번 모두 original-held reopen=would-block, child128bitnativeidentitymatches=true, parent-original-closed/childlive reopen=acquired, terminal reopen=acquired였다. 따라서 duplicate_preserves_lock=false다. 자식은 File.try_clone으로 전달한 실제 stdin 핸들을 ready부터 release까지 보유했다. stdio handoff의 duplicate 원본은 Command drop으로 닫았으며 child original native ID로 전달을 확인했다. 관측 동안 child가 살아있음을 owned Child::try_wait로 확인했다. native fixture 시간은 원시로그 각각26/27/28 ms이며 elapsed에 별도 host 비용은 포함하지 않는다.

source SHA256: 89e8a2ccbe387ee57517d402c582b4db32b3f86581c803ede629eee4ac8d69eb. native tool은2092544B이며 SHA256은 a74e681775d32f7c11c6a5485b385fd7d1a3410fb35841e977734b5617ea4910이며 repeat-results.json에도 저장했다. 실제 부모/자식 PID는 각 raw 로그의 소유 관측값일 뿐 재사용 신호 권한이 아니다. original process handle로 bounded child reap 후 자체 생성 temp fixture를 정리했다.

Windows guardian은 Unix same-OFD처럼 app의 lock을 duplicate만으로 넘겨받을 수 없다. 앱이 durable Intent를 먼저 기록하고 parent lock을 해제한 뒤 guardian이 자신의 핸들로 직접 lock을 획득하고 ACK해야 한다. 이 짧은 gap에도 pending Intent 및 실제receipt 부재로 startup 삭제를 거절해야 한다. guardian lock ACK/Bound commit/Begin 승인 이전 reader Resume는0이어야 한다. concurrent recover가 잠금을 먼저 얻으면 guardian은 bounded 실패/대기 후 tool0로 보존하며 우회하지 않는다. 실제 제품 handoff와 app crash gap 인수는 후속 작업이다.

이 관측은 같은 프로세스의 모든 Windows lock semantics나 독립 guardian 제품완료를 확정하지 않는다. 부모가 잠금 소유 프로세스인 현재 File::try_lock/try_clone/stdio inheritance 경로에 대한 실제 반례다. Cargo/제품Rust/schema/GUI/시스템 설정은 변경하지 않았다.
