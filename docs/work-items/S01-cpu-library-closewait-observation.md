# S01: CPU 목록 CloseWait 관측 원인분리

- 상태: done
- 분류: validation
- 우선순위: P0
- 요구사항: R03, R08, S01
- 담당/수정 범위: diagnostic_cache; 신규 scripts/cpu_library_close_probe.py 및 전용 validation/evidence
- 선행 작업: S01-cpu-library-gui-validation, CPU READY 8fb4a67c27ff89fd883ff6fddaa9136a5fc36c62d74c049665543a513ab06986

## 사용자 결과

실제 목록 Refresh busy 상태를 확정한 후 CloseWait를 관측한다. 미확인 timeout을 성공으로 바꾸지 않는다.

## 계약과 범위

제품/공유 source/Cargo 변경 없음. owned Xvfb/private bus/합성10,000파일/임시 SQLite lock만 사용한다. 사용자 desktop/audio/clipboard 금지. named 노드 사전 조회 후 fresh GetState 단일 call과 실제 AT-SPI DoAction을 사용한다. 각 단계 monotonic timestamp, full-tree duration, lock acquisition과 release를 기록한다. accessible_library.rs/diagnostics.rs/catalog/Cargo/lock sourcehash를 기록한다.

## 인수 조건

실제 Refresh action 성공과 저장 폴더 entry disabled/ReadOnly를 먼저 확인한다. 닫기 요청→busy prompt→StopExit→lock 보유 생존→해제 후 정상 종료를 관측한다. UI 내부 worker ack 직접 관측이 아니므로 외부 생존/종료 증거로 범위를 명시한다. raw 실패 보존, 직접 child 회수와 바이너리 hash 불변.

## 검증과 handoff

이전 helper는 full tree에 최소1초 sleep+최대192 gdbus calls가 있어5초 busytimeout을 소진할 수 있다. 이전 두번째 재시험은 cached 단일 GetState만으로도 timeout되어 full-tree latency만으로 원인을 단정할 수 없다. 새 trial은 단일 호출 시간을 실측하며 full-tree를 lock 중 사용하지 않는다.

## 실제 결과

button Enabled proxy가 pinned AT-SPI mapping 때문에 부적절한 원인을 확인했다. fresh entry GetState로35ms 내 busy를 확인한 뒤 실제 StopExit action, lock 보유 생존, 해제 후116ms 정상종료를 확인했다. docs/validation/cpu-library-closewait-validation.md 및 entry-state/result.json. 내부 StopAck 직접 관측은 아니다. 원 실패 보존/소유 child 종료/hash 불변.
