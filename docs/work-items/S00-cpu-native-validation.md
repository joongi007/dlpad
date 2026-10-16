# S00: CPU 네이티브 후보 IME·무음 reader 검증

- 상태: done
- 분류: validation
- 우선순위: P0
- 요구사항: R08, R11, S00
- 담당/수정 범위: diagnostic_cache; 기존 scripts/accessible_ime_probe.py, scripts/accessible_reader_probe.py, 신규 docs/validation/cpu-native-validation.md 및 docs/validation/cpu-native evidence
- 선행 작업: root의 CPU 후보 fmt/clippy/10 tests/release PASS, SHA28b412773a5fc1434f86d396667a30e7b6d671a8e30affbe1a41cb1ecea7d658

## 사용자 결과

CPU 후보에서 한글 조합·확정·취소·Latin·선택, 실제 접근성 트리와 무음 Orca 이벤트·Tab focus를 확인한다.

## 계약과 범위

source/Cargo/build 변경 금지. 기존 가상화 probe를 재사용하고 새 private Xvfb/bus/temp app-data/media sibling에서 시험한다. 사용자 desktop/audio/clipboard를 사용하지 않는다. GL 부재·메모리 측정은 root 담당. 최초 실제 레이아웃을 확인하고 실패 원시 자료를 보존한다.

## 인수 조건

동일 CPU 바이너리 hash의 실제 PNG/GetText 및 reader 로그를 수집하고 시각판정과 원 collector 상태를 구분한다. reader event handling 성공을 전체 reader usability/음성/3 OS 성공으로 확대하지 않는다. 소유 직접 프로세스 회수와 임시 fixture 제거·끝 hash를 확인한다.

## 검증과 handoff

실제 레이아웃 확인 후 CPU 동일 hash에서 IME 및 무음 reader fresh 검증 완료. 한글 조합 시각/확정/취소/Latin/선택과 실제 reader text/caret/Tab focus 처리 확인. 원 IME collector exit1과 시각판정 review를 구분 보존했다. 종료 후 동일 SHA 확인, source/Cargo/build 및 사용자 desktop/clipboard/audio 변경 없음. 전용 증거: docs/validation/cpu-native-validation.md.
