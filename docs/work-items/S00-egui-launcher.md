# S00: 비교 화면 설정 진입

- 상태: done
- 분류: validation
- 우선순위: P0
- 요구사항: R06, R08, R11
- 담당/수정 범위: 주 담당; app/egui_spike.rs, 본 계약. diagnostic_cache는 별도eguiIMEprobe/증거소유.
- 선행 작업: 현재 S00 감사 FAIL 및 기존 alternative feature

## 사용자 결과

비교 바이너리의 --fixture --settings로 한글 입력 시험 화면에 직접 진입한다.

## 계약과 범위

기존 Rust eframe 비교 화면만 변경하며 제품binary/스택채택을 바꾸지 않는다. 기존 pinned Cargo alternative 의존성과 labelled_by input을 재사용한다. GUI 시험은 ownedXvfb에서만 진행한다.

## 인수 조건

명시설정옵션이 fixture library보다우선하고, 가상화면에서 입력·accessibility 실제증거를수집한다. 빌드/화면성공이IME/스크린리더통과를대신하지않는다.

## 검증과 handoff

비교release빌드및새가상IMEprobe준비중. 제품스택은Proposed유지.

실제 비교release빌드2분36초 성공. --fixture --settings설정창은담당agent가실제Xvfb에서확인했다. 주담당의 별도10,000행library baseline:8809792B/RSS133636KiB/idle0.0%(10.0115초)/960x640/ownedchild종료. 이단일표본은실제스크린리더/IME/장시간성능을대신하지않는다. [증거](../validation/evidence/S00-egui-library-baseline.json). 새라이브러리추가아닌기존lockedalternativefeature를실행했으며제품default는유지했다.

한글누락 재현의원인분리를위해 --fixture --settings --ime-events에서만 메모리IMEtrace를활성화한다. 최신16개/내용80문자상한이며 UI만표시하고 disk/log로저장하지않는다. default제품/후보일반flow에는추적을표시하지않는다. 실제가상화면담당이별도hash로재검증한다.

실제rawtrace에서 Commit글이수신되지만Enabled/Preedit없는TextEdit에반영되지않음을확인했다. --fixture --ime-commit-fallback 실험옵션으로inactiveCommit직전에만Enabled를넣어현재cursor epoch를초기화한다. rawtrace는변환이전입력을유지한다. baseline과같은물리키/선택/취소/라틴입력시험으로가설을검증하며inlinepreedit/실제스크린리더완료로확대하지않는다. 제품default에는적용하지않는다.

대조완료: 동일candidate해시69dd5adf23f019c0b94c6b96d641a03390180549ba270e106b6792831c0d831f에서fallback-off는한글누락, opt-in은exactcommit/취소/영문/선택대체통과. inlinepreedit는양쪽FAIL이므로제품스택은확정하지않는다. [전체대조증거](../validation/egui-ime-validation.md). 다음XIMreference는별도계약으로진행하며이완료는launcher/trace/실험범위만이다.
