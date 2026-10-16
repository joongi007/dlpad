# S00: 명시 XIM callback reference 원인 분리

- 상태: done
- 분류: validation
- 우선순위: P0
- 요구사항: R08, R11, S00
- 담당/수정 범위: diagnostic_cache; 신규 scripts/xim_reference_probe.c·scripts/xim_reference_probe.py 및 이 계약/전용 validation/evidence
- 선행 작업: S00-current-audit/egui-ime 실패, 기존 Fcitx/Xvfb/private D-Bus 도구

## 사용자 결과

동일 Fcitx가 명시 style1026 client에 실제 한글 preedit callbacks를 보내는지 관측해 환경과 native toolkit 입력 경계를 분리한다.

## 계약과 범위

XCreateIC에 XIMPreeditCallbacks|XIMStatusNothing을 명시하고 draw/start/done/caret callback을 finite capture한다. 제품 UI/기존 probes/registry/vendor/Cargo 수정 금지. owned Xvfb/private bus/temp XDG/Fcitx만 사용하고 desktop fallback 없음. cc는 기존 /usr/bin/cc, 필요한 개발헤더는 root 합의 후 프로젝트 전용 공식 패키지 추출만 사용한다.

## 인수 조건

requested/observed style1026, 실제 물리키 gksrmf와 Space commit, boundeddraw trace/한글preedit text·callback순서, reference source/binary hash, process cleanup·timeout을 기록한다. callback success와 제품inline가시성/OS reader 성공을 구별한다.

## 검증과 handoff

[최종 실증](../validation/xim-reference-validation.md): 같은referencebinary의None9.795초/event.window9.792초 두시험exit0. serverDraw한글·commit·취소·Latin통과, nonemptyencodingflags전부0. Start/Caret미관측과style-readbackunsupported별도보존. winit의Start없는Updateguard가다음원인후보다. 기존cc/공식project-localheaders공급hash/cost와AST/CLI/trace상한실패검증완료. 제품stack acceptance는미완료.
