# S10: 정확한 보고서 용량 경계 인수

- 상태: done
- 분류: validation
- 우선순위: P0
- 요구사항: R10, R13
- 담당/수정 범위: metrics_recovery는 /tmp 보고서 cap 감사·독립 회귀 초안만; root는 후속 source/docs/test 게시·전체 native 검증. Cargo/schema 변경 없음.
- 선행 작업: 기존 bounded 보고서 API와 large GUI 근거 읽기, current native source 동결.

## 사용자 결과

정확2MiB 방어 경계와 유효 도메인 보고서의 실제 최대량을 구분하고 초과 자료를 기존파일 변경 없이 거절한다.

## 계약과 범위

DiagnosticReport 공개API/실제 JSON·Markdown preview/export worker 경로를 기준으로 한다. 임의 unchecked field나 외부실제payload·비밀로 합성fixture를 부풀려 유효한제품보고서로 주장하지 않는다. 회귀는 원 TEMP·sentinel·create-only 출력만 사용한다.

## 인수 조건

정확한 byte길이/UTF8·bound+1·실제 export결과와원파일보존, GUI paging8KiB상한 연결 여부를명시한다. 유효API에서정확2MiB자료생성이불가능하면수학적/실제측정근거를기록하고 방어cap시험과도메인인수를분리한다. 단위검증으로GUI/clipboard/reader대체없음.

## 검증과 handoff

현재large validJSON1.84MB/Markdown1.93MB 원 Windows 시험과 초과거절단위시험은있으며 정확2MiB 여부는미확인이다.

root는scalar직접변조를제외한boundedAPI원field/serializer를검토했다. 9MPDfields→11lines·kind길이+7/boolean+2·restrictedversion48/URLs128/로그4096×128와기타65KiB상계를대조했다. 유효exact2MiBGUIfixture로주장하지않는다. 담당파일에engine/reports.rs privatewriter추출과방어bytes회귀를추가했다. exactJSONstring=outerquotes+한글699050→2097152B,EOF·+1 beforecreate/nohook·원sentinel·원Unix/Windowspartial정책을검증한다. 공개API/스키마/의존/정책변경없으며실제native회귀와최신전체검증전완료처리하지않는다.

## 2026-10-05 원 결과 통합 완료

최신 Windows default CPU fmt/clippy/test/release 네 명령 exit0, 현재 소스534개 SHA 불일치0, CPU966c6b798c95113b9f0a191bdb22504742f51efcc030840dce7297bfe308759e/7856640B를 root가 직접 확인했다. 증거 [최신 전체 검증](../validation/evidence/windows-report-cap-current-default-20261005-resume/result.json). 이 작업의 좁은 인수 완료이며 전체 P0 완료가 아니다.

방어 exact2097152B·+1·create-only·write실패 정책 신규2회귀 Windows PASS, 원reports.rs Unix standalone6 PASS. 실제 유효 API의 보수적 상계 JSON1943923B/Markdown2036436B로 exact2MiB 도메인 fixture와 구분한다. 최신 실제 GUI JSON1841855B/Markdown1933102B 저장·원본 SHA·readonly DB Failed1/attempt1/artifact0를 root 검증했다. [원 파일 검토](../validation/evidence/windows-report-cap-large-markdown-reveal-fixed-20261005-resume/root-file-review.json). 최초collector 실패를 보존하며 UIA ScrollIntoView→focus→bounds 검증 순서만 교정했다. Clipboard/reader/peak/CloseWait in-flight는 미검증이다.
