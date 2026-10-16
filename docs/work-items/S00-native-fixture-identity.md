# S00: Windows 병렬 시험 fixture 이름 충돌 수정

- 상태: done
- 분류: bug
- 우선순위: P0
- 요구사항: R10, R11
- 담당/수정 범위: 주 담당 accessible.rs cfg(test) Fixture 및 본 문서; library_identity accessible_library.rs/accessible_diagnostics.rs cfg(test) Fixture 독점
- 선행 작업: Windows S10 native 테스트의 실제 Os183 실패

## 사용자 결과

OS 시계해상도와 관계없이 병렬시험이 각자 소유한 임시 폴더를 사용한다. 제품runtime의동작이나사용자파일은변경하지않는다.

## 계약과 범위

시간/PID만의이름대신프로세스내AtomicU64 sequence를추가한다. create_dir AlreadyExists는최대256회새이름으로재시도하며기존폴더를지우거나덮어쓰지않는다. cfg(test)only 수정. GUIsource snapshot 완료후수정하며productionbinaryhash는재빌드로직접대조한다.

## 인수 조건

기존Windows 실패raw보존, 수정소스의nativeclippy/test/build beforeafter불변 및실제시험통과. Linux기존전체회귀통과. 오류를무시하거나미수정재실행으로성공시도하지않는다.

## 검증과 handoff

기존WindowsS10 Clippy는PASS, 테스트는Os183 AlreadyExists로실패해release미실행. root증거수집종료뒤구현/재검증한다.

최종 실제 검증: Windows freshprefix Clippy6.46초/test15.17초/release30.97초exit0,167passed/0failed/1ignored. 주 담당이현재237source입력SHA를독립대조해불일치0을확인했다. [Windows 원시 결과](../validation/windows-cpu-diagnostics-fixturefix/result.json). 실패원시 windows-cpu-diagnostics는별도보존했다. Linuxfmt/CPUclippy/default193passed/CPU242passed/각0failed5ignored/release34.77초exit0. cfg(test)수정전후productionCPU SHA8fb4a67c27ff89fd883ff6fddaa9136a5fc36c62d74c049665543a513ab06986과10437144B가실제로동일했다. Windows는최초S10release가실행되지않아testfix전같은productionSHA대조를주장하지않는다. 문서208개검사통과.
