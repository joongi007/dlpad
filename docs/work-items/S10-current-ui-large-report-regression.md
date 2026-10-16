# S10: 최신 UI 바이너리의 대형 진단 회귀

- 상태: done
- 분류: validation
- 우선순위: P0
- 요구사항: R10, R13
- 담당/수정 범위: root 기존 scripts/windows_s10_large_report_native_probe.py 실행·새 evidence·본 문서; 하위 원자료 읽기검토만
- 선행 작업: 최신540소스 CPU 전체4명령 및 목록 GUI측정 종료

## 사용자 결과

동일목록 UI전송을 바꾼 최신 바이너리에서도 S10 대형 JSON/Markdown/미리보기 진단을 열고 저장할 수 있는지 검증한다.

## 계약과 범위

원collector를 바꾸지 않고 fresh synthetic fixture/private desktop만 사용한다. 원CPU3f9d8825f1674b1c682094506f8774df208494dc10ccf1e0655f62d06b8e3667과fixture/sourceSHA를 고정한다. clipboard/실제account/사용자Documents/visible desktop은 시험하지 않는다. native 작업은 root 직렬 실행한다.

## 인수 조건

large-json/large-markdown/large-preview 세case의 실제 원export/DB/이미지·Job 종료/private/input 불변·원소스/바이너리 불변 대조. 수집기PASS만으로인수하지 않는다. 보고서 상한과 미검증clipboard를 명시한다.

## 검증과 handoff

실행 준비 중. 앞선 snapshot의 성공을 최신 검증으로 대체하지 않는다.

`python3 /tmp/dlpad-current-large-report-suite.py`의기존nativeprobe3case가exit0/source540불변/CPU3f9불변/원Job자연0/private/input을통과했다. root가JSON1841855B/Markdown1933102B 원exportSHA·128logs×4096bytes/8MPD×256nodes·DB integrity/Failed-noartifacts·sourcecookie/sentinel/calls를대조했다. 미리보기first/next8192B,last236쪽7982B/Next비활성을원JSON+실제PNG로확인했다. 원REVIEW_REQUIRED는유지하고root/독립검토를새evidence windows-current-large-reports-20261005-continue에저장했다. 정확2MiB/이번peakmemory/clipboard/physicalreader·IME는미검증이며일반site실패코드와stressconstructor exit를동일시하지않는다. 본최신회귀범위done이며전체P0완료가아니다.
