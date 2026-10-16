# S00: CPU 최신 근거와 README 범위 갱신

- 상태: done
- 분류: documentation
- 우선순위: P0
- 요구사항: R01, R07, R08, R11, R13
- 담당/수정 범위: metrics_recovery — README의 CPU 후보 현재 검증 범위 구역만; cpu-paged-cost.md; 신규 windows-cpu-paged-preview.md; 전용 evidence review
- 선행 작업: Linux1669 및 Windows250test 원시 결과/inputs/release

## 사용자 결과

실제 검증된 기능과 OS별 미검증 게이트를 현재 근거에 맞게 구분한다.

## 계약과 범위

문서읽기계약과 기존 Accepted/native 및 Proposed/최종stack 상태 유지. P0-status/제품source/Cargo/sharedschema/다른 README구역 변경금지. 하위spawn없음.

## 인수 조건

Windows rawlogs에서250pass/0fail/1ignore와258inputs각전후현재SHA 독립대조, 실제 exe7244288bytes/SHA확인. Linuxidle JSON 검토, plan/tool/history/background/maxvalidreport 링크와 각snapshot범위, 최종OS/stack/릴리스미완료 명시.

## 검증과 handoff

세 Windows rawmanifest258개 각각 before/after/current SHA 일치, rawtest250/0/1 합산, 세exit0/desktop unchanged 및 실제exe7244288bytes/a094…SHA를 독립확인했다. Linuxidle 원시수치 및 링크별snapshot을 구분해 README 지정구역만 갱신했다. 전용Windows 문서/independent-review.json과cost검토를 기록했다. 제품/source/Cargo/P0-status/기타README구역 수정없음. 다른OSGUI/MSVC/전체release/P0완료를 주장하지 않는다.


검토 완료 뒤 root의 download.rs 테스트-only cfg 이동으로 이전Windows manifest와 현재파일1개가 달라질 수 있다. Windows문서에 변경전검토 시점 및 이전250PASS snapshot을 명시했으며 추가Windows재검증은 root 후속이다. 제품 release 동작/기존binarySHA 동일이라고 기록한다.
