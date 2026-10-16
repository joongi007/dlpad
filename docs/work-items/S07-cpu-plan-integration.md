# S07: CPU 원본 URL 생략 실행 계획 연결

- 상태: done
- 분류: feature
- 우선순위: P0
- 요구사항: R07, R08
- 담당/수정 범위: root; accessible.rs/두entry/catalog.rs 및 본 문서·통합검증
- 선행 작업: S07-cpu-plan-preview; typed options

## 사용자 결과

입력한 품질·구간·고급 옵션의실제argv를원본URL전체생략한JSON으로확인한다. 입력수정시열린미리보기는현재옵션으로다시계산한다.

## 계약과 범위

원coreplan만사용하고shell/외부실행/IO없다. finite256KiB builder 실패는고정centralcatalog문구. preview접힌기본에는계산하지않고열린경우만현재typed옵션을확인한다. Unicode표시실패는lossy경로로실행계획을오해하게하지않는다. Cargo/schema/dependency추가없음.

## 인수 조건

URL/query/auth비노출·literalargv/한글/공백/구간보존·실패회귀·실제격리GUI에서옵션수정반영·invalid거절·닫힌기본비용검증. 원래다운로드시작/재개옵션은동일core계약을유지한다.

## 검증과 handoff

연결후diagnostic페이지화와함께rootCPUclippy/unit/release및실제GUI를진행한다. 기존snapshot결과를새compiledsource성공으로표현하지않는다.

## 인수 결과

1669 snapshot에서 CPU unit71PASS/clippy/release를 실제 실행했다. [실제 GUI](../validation/cpu-plan-validation.md)는 literal argv·URL 전체 생략·720/1080 변경·잘못된143 품질·미실행 sentinel/DB 부재를 확인했다. [기본 비용](../validation/cpu-paged-cost.md)은 열린 큰 미리보기 peak와 구분한다. keyboard/pointer toggle는 별도 실제 시험으로 확인했고 펼침 상태 metadata 수정은 S00-collapsing-accessibility-integration 후속이다. Windows native 당시250PASS/1ignored; GUI/macOS는 미검증. P0 전체/최종 stack 채택을 뜻하지 않는다.
