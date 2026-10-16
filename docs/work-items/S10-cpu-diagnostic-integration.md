# S10: CPU 진단 자료 편집·첨부·저장 연결

- 상태: done
- 분류: feature
- 우선순위: P0
- 요구사항: R13, R08, S10
- 담당/수정 범위: 주 담당 accessible.rs, accessible_main.rs, accessible_cpu_main.rs, catalog.rs, 본 문서; library_identity accessible_diagnostics.rs 독점
- 선행 작업: 실패 Attempt 보고서, reports create-only export 및 로컬 MPD 정제

## 사용자 결과

실패 보고서에서 Markdown/JSON 선택·로그 편집과 제외·로컬 MPD 구조 첨부와 제외·명시 복사·새 파일 저장을 제공한다. 첫 릴리스 필수 P0 범위이며 외부 AI/계정/네트워크 전송은 없다.

## 계약과 범위

새 panel은 Box report 단일소유와단일I/O worker/channel1 및2MiBcachedpreview를사용한다. root는 실패완료의report를panel로옮기고worker poll/CloseWait/새다운로드startgate에연결한다. input32KiB/MPD1MiB/CreateOnly기존정책유지, 자동cache없음. source변경은진행중LibraryGUI snapshot 수집끝신호후수행한다. 전체reportcontrols는스크롤로작은창에서도접근가능하게한다.

## 인수 조건

fmt/clippy/실제7panel실패회귀/기존Candidate·host·library회귀/release 및실제privateGUI복사·편집·첨부·저장·실패보존. UIthread I/O/unfinishedjoin 금지, 종료는정지ack후완료. macro/finalstack/외부전송인수와구분한다.

## 검증과 handoff

신규panel구현·ownerfmt완료이며아직module연결/컴파일전이다. 기존snapshot의copy 성공은새전체진단UI성공을대체하지않는다.

현재진단panel을Candidate보고서소유권과poll/CloseWait/startgate에연결하고중앙catalog6문구를추가했다. 첫CPUclippy16.80초/entry27tests0.22초/release37.35초exit0,10437144B SHA8fb4a67c27ff89fd883ff6fddaa9136a5fc36c62d74c049665543a513ab06986. 직전Librarybinary보다77600B증가,새Cargo 의존성/schema변경없음. owner신규7실패경로를root시험에서실제검증했다. Linux기본193/CPU242tests각0failed5ignored,Windowsfixturefix167passed1ignored. cfgtestfixture충돌은 [별도회귀](S00-native-fixture-identity.md)로수정했다. Windows현재입력237개root독립SHA대조불일치0. 실제진단GUI흐름은guardian_fault전용harness로검증중이며작업in-progress유지.

## 실제 Linux 진단 수직 기능 통합 인수

[최종GUI](../validation/cpu-diagnostics-validation.md)에서새8fb4binary의actualFailedAttempt→MD/JSONprivateclipboard→로그3/수정재정제2/개별제외1/전체제외0→MPD첨부1/개별제외0/재첨부1→UnsafeXml/TooLarge거절/원본보존→전체제외→create-only저장/충돌기존bytes보존→CloseWaitexit0를확인했다. root가관련11JSON과SQLite를직접읽어로그/첨부개수와생략reason·secret누락·Failed/Attempt1/noartifact/integrityok를대조하고attached.png를직접확인했다. editor에는사용자가입력한합성secret가남지만보고서export에는재정제되어없는것을구분한다. 이전하네스라벨/정리순서/콜론으로전체log생략fixture실패는raw보존했다.

boundedLinux S10수직기능은완료한다. 2MiB최대보고서GUIstress·Windows/macOSS10전체GUI/clip·전체P0/최초릴리스완료는별도게이트다. 새CPU대기baseline은RSS50788KiB(49.6MiB)/idle0%10.324초 [직접실측](../validation/accessible-performance/cpu-diagnostics/result.json)이며열린대형보고서peak비용이아니다.
