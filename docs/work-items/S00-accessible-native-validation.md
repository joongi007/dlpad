# S00: 실제 접근성 후보의 IME·무음 reader 검증

- 상태: done
- 분류: validation
- 우선순위: P0
- 요구사항: R08, R11, S00
- 담당/수정 범위: diagnostic_cache; 신규 scripts/accessible_ime_probe.py, scripts/accessible_reader_probe.py, docs/validation/accessible-native-validation.md 및 전용 evidence
- 선행 작업: root의 frozen dlpad-accessible SHA256 0006783276df0485f25bfeec0af805389cb81ac0e35d9b25116243cf9c823dd1, 후보 다운로드 slice/5 tests/alternative clippy/release PASS

## 사용자 결과

실제 다운로드 엔진을 포함하는 접근성 후보에서 한글 조합·확정·취소·Latin 전환·선택 및 AT-SPI/키보드/실제 무음 Orca 이벤트 처리를 확인한다.

## 계약과 범위

제품/engine/Cargo/build 및 다른 담당 accessible_download_probe.py 수정 금지. 기존 probe는 read-only 재사용. owned Xvfb/private D-Bus/AT-SPI/temp data/media sibling/XDG/GSettings memory만 사용한다. 사용자 desktop/clipboard/audio에 접근하지 않는다. Clipboard 대신 AT-SPI GetText로 결과를 읽는다. 프로젝트 전용 기존 IME/Orca 도구만 재사용한다. 새 child agent를 만들지 않는다.

## 인수 조건

후보 실제 레이아웃 inspect 후 정확한 SHA/PNG/원 JSON을 보존한다. 제품 IME와 spike 성공을 구분한다. Orca 실제 이벤트 처리와 전체 reader 사용성/음성을 구분한다. 원 실패는 유지하고 모든 소유 직접 프로세스를 회수한다. 다른 OS·전체 제품 parity는 미검증으로 남긴다.

## 검증과 handoff

최종 SHA55ecdbc89c4eb42c4e898e939e7ca164e94ed5359848a903c7d58d77a452401f에서 실제 한글 조합·확정·취소·Latin·선택과 AT-SPI/무음 Orca text/caret/Tab focus 처리 재실증 완료. 원 좌표/collector 실패는 삭제하지 않고 review로 구분했다. clipboard/desktop/audio/제품/Cargo/build 변경 없음. 전체 reader 사용성·3 OS·제품 parity는 미검증. 증거와 실행 결과: docs/validation/accessible-native-validation.md.
