# S03: reader drain 최신 native terminal 회귀

- 상태: done
- 분류: validation
- 우선순위: P0
- 요구사항: R03, R08
- 담당/수정 범위: stress_recovery; scripts/reader_drain_terminal_regression.py 및 이 문서
- 선행 작업: root guardian reader-drain 수정 및 실제 engine lib201 통과

## 사용자 결과

같은 고정 driver/helper로 실제 사용자 manager API terminal 및 socket EOF를 재검증한다. 이전 22case 결과를 최신 source의 성공으로 재사용하지 않는다.

## 계약과 범위

기존 terminal_user_service_probe.py는 수정 없이 subprocess 호출한다. 새 wrapper는 driver/helper를 별도 owned temp에 immutable mode0500으로 복사하고 source 입력과 binary digest를 전후 기록한다. 미연결 reader_journal.rs는 컴파일 입력에서 제외하고 이유와 SHA를 별도로 기록한다. 기존 manager 설정, unit, 장기8h 시험은 변경하지 않는다. fixture 및 이번 실행이 생성한 unit만 기존 probe 소유권 계약으로 정리한다. 실제 OS 실행/Cargo는 root 담당이다.

## 인수 조건

기존 API17/raw3/fake2 총22case: 실제 GC/GCLC 및 socket/pipe EOF, typed unconfirmed, exact PID/start/pidfd, cgroup/unit 소실, sentinel/FD 및 owned temp 정리를 확인한다. source/binary 입력은 전후 불변이어야 한다. wrapper의 성공만으로 별도 source 전체 통합 또는 다른 OS를 완료로 표현하지 않는다.

## 검증과 handoff

Root 준비 명령: `cargo build -p dlpad-engine --bin dlpad-process-guard --locked`, 이어 `cargo test -p dlpad-engine --test terminal_user_service --no-run --locked --message-format=json`. compiler-artifact executable을 driver로 사용한다. 신규 wrapper: `python3 scripts/reader_drain_terminal_regression.py DRIVER HELPER /absolute/project/docs/validation/evidence/NEW`. wrapper 작성 및 Python AST parse 통과. SHA `932c43908bf170aa344c30ed210802b90566f65157877e21daf2d11170dc9f9f`로 동결 인계했다. Cargo/native OS 시험은 이 담당자가 실행하지 않았으며 root 실행 및 독립 raw 검토 대기다.

## Root 실제 실행·독립 검토

위 helper/driver Cargo 빌드와 신규 wrapper 실제 실행 exit0. 원시22case PASS, raw exit0/42/126의 정확 RGCLC+EOF, 소유 PID/start78개 소멸, cgroup14개 소실, sentinel 보존을 root가 재확인했다. [검토 증거](../validation/evidence/reader-drain-native-terminal-first/root-review.json). 현재 cookie gate/recovery·GUI·다른OS·장기8h는 이 검증 범위 밖이다.
