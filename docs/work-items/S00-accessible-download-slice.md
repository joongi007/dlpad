# S00/S02: 접근 가능한 native 후보의 실제 다운로드 흐름

- 상태: done (bounded Linux candidate vertical slice)
- 분류: feature
- 우선순위: P0
- 요구사항: R04, R08, R13, S00, S02, S10
- 담당/수정 범위: 주 담당 Cargo/accessible_main.rs/통합검증; diagnostic_cache 신규 accessible.rs만, catalog read-only
- 선행 작업: Iced AT-SPI tree 부재 및 egui 실제 tree/Orca 이벤트 관측

## 사용자 결과

접근 가능한 egui native 후보에서 기존 Rust 엔진의 URL 다운로드·진행률·중단·실패 보고서·완료 artifact를 실제 연결한다.

## 계약과 범위

기존 제품을 제거하지 않고 alternative feature의 dlpad-accessible 별도 검증 실행 파일을 제공한다. core/엔진 정책·SQLite·원래 Job/Attempt·artifact검증을 재사용하며 egui worker는 I/O를 UI thread에서 하지 않는다. guardian entry/register/instance guard는 root entrypoint가 공유 계약을 유지한다. 완료/실패 capacity1 이벤트와 최신 ProgressFeed, 하나의 owned worker/cancel/pause만 허용한다. 라벨은 기존 catalog를 재사용하고 신규문자열은 root에 요청한다. implicit 원격전송/도구설치 없음.

## 인수 조건

URL/tool/root typed옵션 검증→worker→기존Repository state/완료등록→정제보고서/명시복사까지 actual합성fixture로 연결한다. pause/cancel/실패에는 중복worker/start가 없고 소유 프로세스 종료ack 전 성공처리하지 않는다. native한글/AT-SPI/keyboard를 기존probe로 후보에 재검증한다. 이 수직 기능은 전체 UI parity/최종3OSstack채택을 대체하지 않는다.

## 검증과 handoff

별도후보 구현 진행. 현재 Iced 개발제품은 유지하며 실제 접근성 인수부족을 문서화한다.

## 실제 GUI 일시 중단 회귀

최초 후보 SHA000678...의 actual GUI에서 Pause 버튼 조작 후 Cancelled가 2회 재현됐다. worker helper의 first-intent 테스트는 통과했으나 UI 호출부가 pause=true를 기록한 뒤 request_stop(false)로 덮어썼다. 주 담당이 frozen accessible.rs의 해당 호출만 request_stop(true)로 수정한다. 기존 실패 증거를 보존하며 새 SHA에서 actual Pause→Paused→sameJob/newAttempt 재개 및 success/failure/close를 재검증한다. helper 시험만으로 GUI 분기 인수를 대체하지 않는다.

## 주 담당 실제 통합 판정

Pause수정 hash55ecdbc89c4eb42c4e898e939e7ca164e94ed5359848a903c7d58d77a452401f에서 실제 success/failure/pause/close4개 PASS. 주 담당이 원시SQLite4개 read-only integrity_check=ok, Downloaded/Failed/Downloaded(2Attempts)/Cancelled와 artifact수를직접확인했고1423B보고서에서fixture비밀/미디어URL제거를대조했다. [GUI 증거](../validation/accessible-download-validation.md), [IME/실제 무음 reader](../validation/accessible-native-validation.md)를 따른다. 실제 한글preeditPNG직접view와Orcaentry→다운로드focus로그를대조했다. 후보5tests/clippy/release PASS, 전체features workspace/release도PASS(숫자아래통합상태기록). 수직slice만완료이며 메모리예산 FAIL·UIparity·3OS·최종stack채택/전체P0는남는다. 이후CPUhost용sharedUI추출은 별도snapshot으로재검증한다.
