# S00: 연결 입력의 라벨·오류 문맥 함께 공개

- 상태: completed
- 분류: UX
- 우선순위: P1
- 요구사항: R01, R11, R13
- 담당/수정 범위: root — crates/app/src/accessible_metrics.rs, 본 문서; 하위 에이전트는 해당 제품 파일 수정 없음
- 선행 작업: source655 현재 native/진단 작업 종료·검증 결과 보존

## 사용자 결과

확대 화면에서 잘못된 연결 이름을 입력할 때 입력 제목과 오류 설명을 함께 읽는다.

## 계약과 범위

새 source655의 실제 Dark200/800 PNG에서 오류 설명은 완전히 공개되었지만 인터페이스 라벨은 body 위로 일부 잘렸다. 현재 input+error union 공개를 label+input+error union으로 확장하는 좁은 수정 후보다. 원ID/description/문법·Apply refusal/worker 선택·현재 표시 문구·Cargo/schema를 유지한다. 현재 소스의 실패/성공 증거는 덮어쓰지 않는다.

## 인수 조건

실제 ScrollArea Context에서 라벨·입력·오류의 전체 영역을 확인하고 수정/빈값/Refusal 상태를 보존한다. 새 Windows 전체검사 후 Dark200/800 실제 입력 및 PNG에서도 helper 수동 scroll 없이 세 영역을 확인한다. 물리 입력·reader는 별도 미검증으로 유지한다.

## 검증과 handoff

현재는 발견/후보 단계이며 source에 적용하지 않았다. 근거: ../validation/evidence/current-ui-feedback-network-syntax-dark200-800-20261013/gui/network-invalid-before-clear.png. 현재 시험은 input+error 전체 공개를 실제 확인했지만 이 문서의 세 영역 공개 인수는 아직 미완료다.

root가후보를현재소스에통합했다. 이전v8의source2/기본binary3파일은 /tmp/dlpad-ui-v8-before-label-titlebar-20261013에원SHA/bytes와함께보존했다. 새Windows whole fmt/clippy/test/release는진행중이며아직통과보고가아니다. 공유Cargo/lock/schema변경없음.

### 통합 whole-v2 결과

입력 제목/입력칸/오류 설명 전체의 union을 공개하도록 실제 scroll rectangle을 수정했다. 실제 Context test가 세 영역의 viewport 및 순서를 검사한다. Windows fmt/clippy/test/release1,284/0/30 실제 통과 후 주 담당이4원 로그/source655 hash map/3artifact/GNU/lock을 독립 대조했다. 최신 Dark200/Light100 native label3 시험은 별도 current-only witness19개를 준비하여 진행한다. 이전 v8 입력+오류 성공을 라벨 공개 성공으로 소급하지 않는다.

최신 compact CPU0093의 Dark200800 및 Light100960 실제 input/refusal/clear/Off/Auto 시험과 원로그·source/pins 독립검토·before-clear PNG 직접검토 모두PASS이다. scoped 인수조건 완료이며 전체UI·물리입력·reader·OSDPI 인수는 별도이다. [밝은 화면](../validation/evidence/current-compact-network-label-input-error-light100-960-20261013/root-review.json), [전체 검사](../validation/evidence/windows-msvc-ui-compact-network-error-whole-20261013/root-review.json). 원f15 Dark200 실패는 보존한다.
