# S03: 전용 EOF keeper로 parent proof 수명 확보 실험

- 상태: done (합성6case 실제 실험, 제품 통합 별도)
- 분류: spike
- 우선순위: P0
- 요구사항: R04, R08, R11
- 담당/수정 범위: stress_recovery; 신규 scripts/cookie_dependency_holder_probe.py 및 이 문서와 전용 evidence
- 선행 작업: SWU runtime setter 제한 및 RAE service 초기 identity 전 실패 잔여물 감사

## 사용자 결과와 계약

reader RAEfalse와 auxiliary SWUtrue parent에 parent 밖 app.slice의 keeper를 연결한다. keeper Requires/After parent, reader Wants/After keeper를 같은 transaction에 구성한다. keeper는 전용 stdin EOF까지 기다리며 writer는 probe owner만 CLOEXEC 소유한다. reader나 manager에 writer를 전달하지 않는다. 합성2초 reader의 정상0/exit7/자기SIGTERM 종료 뒤 원 종료값/pidfd/native parent strict empty를 확인하고 keeper EOF 후 자동 GC를 요구한다. identity capture 전 owner bus/writer 종료도 별도 실제 case로 검증한다. live/pending 합성 sibling 보존도 별도 case다.

## 인수와 정리

parent 직접 Stop0. 모든 keeper/reader/sibling 정상 또는 지정 실패 종료와 unit GC/FD 복귀 확인. parent 밖 keeper는 parent direct procs를 채우지 않아야 한다. keeper에는 시험 전용12초 RuntimeMax와 reader3초 상한만 적용하며 제품8h 정책 채택 아님. 실패 시 알 수 없는 Invocation의 unit Stop 금지; 가능한 경우 own identity/native를 확인한 잔여물만 root 계약에 따라 정리한다. 기존8h/foreign units/global setting/Cargo/Rust 변경 없음.

## 검증과 handoff

`python3 -m py_compile scripts/cookie_dependency_holder_probe.py` exit0. 실제 `python3 scripts/cookie_dependency_holder_probe.py /mnt/d/project/dlpad/docs/validation/evidence/cookie-dependency-holder-first` exit0, 6/6 PASS. source SHA256 `76fb457337f92d876015e7ac3b6aa4f782c4bc0cfa3348f9f877316f293ebe44` 및 readonly 재사용 helper4개 SHA는 result.json에 고정했다. Rust/Cargo 및 기존 scripts 변경 없음.

success1/0, exit7 1/7(Result exit-code), 자기SIGTERM2/15를 원 reader terminal 값으로 확인했다. keeper는 app.slice의 별도 cgroup에 있으며 parent descendant가 아니다. keeper가 살아 있는 동안 reader natural terminal/pidfd 종료 이후 held/fresh-openat populated0+빈procs 동시 proof를 확인했다. keeper code는 동일 pipe native identity를 가진 모든 inherited FD가 readonly인지 검증하며 원 keeper 결과exit0을 요구한다. writer는 owner pipe2 CLOEXEC이며 manager properties에는 readFD만 전달했다.

keeper EOF 뒤 keeper exit0/pidfd 종료, reader/keeper/parent GetUnitENOENT 및 parent cgroup 부재를 확인했다. 정상 parent Stop0, PID signal0, 모든 case FD4→4. live sibling은 active 및 동일 Invocation/PID, pending sibling은 activating 및 동일 Job7294를 keeper EOF 전후 유지했다. sibling natural exit0 후 자동 parent GC다.

preidentity-owner-exit case는 StartTransientUnit 성공 직후 GetUnit/Invocation 조회 전에 owner의 writer와 같은 bus를 닫았다. 별도 읽기 observer로 reader/keeper/parent GetUnitENOENT를 확인했다. 해당 case는 초기 native identity를 의도적으로 확보하지 않으므로 원 PID/start·parent native proof까지 확인했다고 주장하지 않는다. 시간3.2초 후 exact names GC 확인이며 failure fallback Stop을 실행하지 않았다.

이 결과는 합성 EOF holder 후보의 실제 manager 실험 근거다. keeper 보호와 원 서비스 invocation/FD single-writer ownership을 제품 guardian에 binding하는 계약, I/B receipt 및 앱 kill/crash 통합, helper resourcefail 및 실제 외부 queued service 인수는 별도다. 합성 keeper12초/reader3초 RuntimeMax는 시험 상한이며 제품8시간 정책을 변경하지 않는다.

root collector 리뷰 후 pending Job 조건을 빈 값뿐 아니라 첫 token0도 거절하도록 한 조건만 보강했다. 첫 원시의 Job7294는 실제 nonzero였으나 기존 source assertion 자체는0문자열을 거절하지 못했으므로 이를 명확히 구분한다. py_compile 실제exit0. 새 source frozen SHA256 `cc02801077d79ff105233f06ed60868075d078bc007c90a4c2be798e127adb71`; 이전first원시/source76fb snapshot은 그대로 유지한다. 새 snapshot 실제 실행은 root 독립 검증 대기이며 담당 native 재실행/Rust변경은 없다.

## 보강된 snapshot root 독립 실행

root session36028의 `evidence/cookie-dependency-holder-root-first/result.json`을 직접 읽어 source cc02801077d79ff105233f06ed60868075d078bc007c90a4c2be798e127adb71 및6case true를 확인했다. root는6개 원시의 FD4→4, parentStopfalse/signalsfalse, held/fresh strict empty, preidentity exactunitGC, pending nonzero/sameJob을 독립 assert하여 exit0을 보고했다. 본 담당의 첫 실행과 root의 보강 snapshot 실행을 구분한다. 제품 통합은 아직 없다.

## 제품 keeper 인터페이스 제안 (합의 전 구현 없음)

root guardian.rs 담당은 내부 marker `--dlpad-internal-cookie-keeper`와 추가args0 계약을 제안했다. stdin0이 owned readonly pipe인지 확인하고1byte bounded read를 반복해 EINTR만 재시도, 실제EOF면 exit0, 데이터1byte면 고정 오류로 거절한다. keeper는 cookie/capability/receipt/control FD나 실제 도구 argv를 받지 않는다. EOF writer의 유일 소유와 parent dependency를 나타내는 수명 보조이며 keeper 단독을 native empty receipt 권한으로 사용하지 않는다.

user_service.rs 후속 담당 제안은 gated-only pipe2(O_CLOEXEC)로 reader와 분리된 keeper stdin read-end와 launcher-only writer를 만들고, 기존 primary reader+auxiliary parent transaction에 auxiliary keeper를 함께 넣는 것이다. reader는 RemainAfterExit=false 및 Slice=parent, Wants/After=keeper; keeper는 Typeexec/RAEfalse/Requires+After=parent 및 Slice=app.slice; parent는 SWUtrue다. ExecStartEx는 fixed current helper path와 marker만이며 별도 shell/env 조합은 없다. 부모 native proof 안에 keeper PID가 들어가면 안 된다. keeper에는 IgnoreFailure로 원 실패를 숨길 이유가 없다. 기존 nongated 경로는 변경하지 않는다.

writer는 launcher의 OwnedFd 하나로 유지하고 manager message/guardian/tool/receipt 전달에서 제외한다. manager에는 readFD만 표준입력 property로 복제한다. StartTransientUnit reply 뒤 launcher read-end를 닫고 writer는 receipt durable sync와 native identity 재검증까지 유지한다. 동기 process에서 keeper FD가 나중 tool exec에 상속되지 않도록 CLOEXEC/관리 FD 전부의 실제 inherited-FD fixture 확인이 필요하다.

정상 순서는 원 reader terminal ExecMainCode/Status+pidfd 확인→parent held/fresh strict empty→durable receipt write→writer close→정확 keeper terminal 확인/Unref→reader Unref→parent Unref다. parent 직접 Stop은 없다. live/queued sibling은 parent의 별도 dependency로 남으며 keeper EOF 때문에 Stop해서는 안 된다. keeper가 실패하거나 writer가 일찍 닫히면 missing parent proof를 성공 receipt로 대체하지 않는다. 초기 identity 확보 전 오류는 launcher writer/bus 해제와 guardian control shutdown으로 자연 종료시키되 identity없는 unit 추정 Stop은 없다.

launcher 자체SIGKILL/OSCrash에서 keeperEOF는 parent 수명을 끝낼 수 있으므로 durable receipt가 없는 same-boot 상태를 삭제 가능으로 만들지는 않는다. 현재 영구복구 정책과 I receipt shape는 root 별도 계약을 유지한다. keeper 수명에 시험용12초 RuntimeMax를 제품에 그대로 넣지 않으며 실제8시간 이상 작업을 제한하지 않는다. 제품 acceptedstart 뒤 resourcefail, keeperEOF전후 pending/live foreign sibling, byte입력거절, RDONLY/CLOEXEC/inheritedFD, success/failure/signal/cancel/workerKill receipt 회귀를 root helper 재빌드 후 실제 인수해야 한다. 이 문서 단계에서는 Rust/Cargo/native 추가 실행 없음/sourcefreeze 유지다.
