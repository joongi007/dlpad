# S10: Windows 대형 유효 진단 보고서 private GUI 인수

- 상태: in-progress
- 분류: validation
- 우선순위: P0
- 요구사항: R10, R13, R01
- 담당/수정 범위: root 통합·native 실행·독립 검증·이 문서와 scripts/windows_s10_large_report_probe.cs 및 scripts/windows_s10_large_report_native_probe.py. diagnostic_cache는 /tmp 초안만 담당하며 같은 파일을 동시에 수정하지 않는다.
- 선행 작업: root Windows collector 종료 및 frozen binary SHA 확인; 기존 S10 standalone private GUI helper/합성 cookie tool; root native compile·GUI·원 SQLite/파일/PNG 검토.

## 사용자 결과

Windows 제품의 실제 대형 보고서 preview paging·새 파일 저장이 전체 화면/문자열을 UIA에 무제한 노출하지 않는지 확인한다. clipboard/Paste/visible desktop을 사용하지 않는다. 현재는 seam 읽기 감사 및 재현 draft 준비이며 인수 완료가 아니다.

## 계약과 범위

새 Rust hook은 필요하지 않다. `accessible.rs:252`의 기존 `--fixture` AND `DLPAD_TEST_REPORT_STRESS=1`는 실패 보고서 수신 시 `accessible_report_fixture::stress_report()`로 교체한다. 실제 synthetic tool 실패→원 Job Failed/Attempt persist는 그대로이고 report payload만 고정 fixture로 대체된다. 일반 제품은 flag 없음이면 활성화되지 않는다. 외부 JSON/보고서 파일을 임의 역직렬화하는 hook을 추가하지 않는다.

fixture는 공개 DiagnosticReport add_log/add_mpd_attachment API로 logs128×4096B, attachments8×nodes256, 유효 MPD 깊이62를 만든다. 기존 Linux full-10은 JSON1,841,853B/Markdown1,933,100B이며 동적 report ID/time 때문에 몇 bytes 달라질 수 있다. 이는 유효 상한 근접 synthetic fixture이지 정확 최대값 수학적 증명 또는 정확2MiB payload가 아니다. `MAX_REPORT_BYTES=2MiB`는 defensive cap이고 초과보고서거절 단위 시험과 구분한다. 인증 실패의 단일 omitted 로그를 stress fixture로 주장하지 않는다.

## 인수 조건

1. 기존 owned root/data/media/source.cookies/cap/mode synthetic fixture를 유지하고 mode metadata-failure(exit7) 호출이 실제 발생한 calls.jsonl을 root가 확인한다. fixture 교체 후 보고서 stage Metadata/exit1은 stress_report의 값이며 실제 tool exit7과 혼동하지 않는다. 원 Job ID/Failed/attempt/options snapshot은 GUI 전후 SQLite에서 확인한다.
2. fresh GUI 한 case씩 `large-json`, `large-markdown`, `large-preview`. 각 worker90초 유지, create-only export≤2MiB·실제 sync/worker return/saved label 확인. JSON logs/첨부/node counts·개별4096Bytes 및 삭제된 URL·source/sentinel SHA 보존을 검사한다. Markdown은1.9–2.0MB 범위·fixed fields 확인. 둘을 한90초 시나리오에 누적하지 않는다.
3. preview는 실제 readonly `진단 자료 미리보기` text role만 읽고 page≤8192UTF8 bytes를 확인한다. 처음/다음 페이지 실제값 변경, 마지막 페이지는 UIA page control 실제값/총 페이지 정보로 이동하거나 별도 bounded case로 확인한다. 200여 번 자동 Next 클릭을 무한 반복하지 않는다.
4. root가 새 binary/collector 원 native IDs·Job 및 자연exit0/desktop close/input desktop 불변을 검증한다. 보고서 root-aware privacy와 파일 SHA/SQLite job invariants를 직접 대조한다. clipboard 인수는 별도 차단 상태이다.
5. loaded report 비용은 root 별도 원 Job working set/CPU 구간 수집이 필요하다. 파일 크기나 일반 idle baseline을 stress peak RSS로 표현하지 않는다. UIA TextPattern 전체2MiB 요청을 하지 않는다.

## 검증과 handoff

현재 기존 source/API와 Linux 실제 raw만 읽었다. root가 기존 C# Main에 owned `DLPAD_TEST_REPORT_STRESS=1` 환경 및 세 case를 연결해 native compile할 수 있는 draft를 준비한다. 프로젝트 제품 hook 변경은 없음. 정확2MiB GUI/실제 clipboard/대형 export busy CloseWait/Windows stress 비용은 아직 미검증이다.

완성 staging `/tmp/windows_s10_large_report_probe.cs` SHA256 `f019db068234fa590c316ff7dcc5226774cc16bf14c4aa3735648011fda2c504`는 기존8argv 계약의 scenario를 `large-json|large-markdown|large-preview`로 제한한다. Main의 owned fixture child 환경에 기존 stress flag만 더하고 VerifyDiagnostic을 해당 한 case로 교체한다. JSON/Markdown 각각 fresh GUI 한 번에서 실제 save/worker return 뒤 정확 크기·SHA·counts를 확인한다. preview는 첫/다음 실제 readonly TextEdit와8KiB UTF8 cap/내용변경/PNG를 확인하며 마지막페이지 false를 명시한다. 실제 tool exit7은 calls/SQLite 원시로 root 검토해야 하고 stress 보고서 exit1을 실제 tool 종료로 해석하지 않는다. native compile/runtime은 수행하지 않았다. root publication/runner scenario 연결과 frozen SHA 인수 후 실행한다.

root는 C# 최종 SHA를 확인하고 scripts/windows_s10_large_report_probe.cs에 통합했다. scripts/windows_s10_large_report_native_probe.py는 기존 bounded S10 묶음 입력 runner의 source 선택과 scenario 목록만 바꿨다. 원 90초 worker·150초 outer와 source·exe·collector pin을 유지한다. 첫 large-json fresh 실행을 시작했으며 native 결과는 아직 미확정이다.

## 2026-10-05 실제 결과와 마지막 페이지 후속

large-json/large-markdown/large-preview 세 fresh private GUI 시험이 통과했다. 각 evidence/windows-large-report-{json,markdown,preview}-20261005-resume/root-review.json에서 원 source snapshot·파일 SHA·Failed/단일 Attempt/Artifact 없음·영구 Job 쿠키 없음·version/metadata 실제 호출·원 source/sentinel 보존·Job0/private desktop 정리를 독립 확인했다. JSON1841855B, Markdown1933102B이며 두 자료를 역직렬화해 logs128×4096B/MPD8×256nodes를 확인했다. PNG도 주 담당이 직접 확인했다. 처음/다음 preview 각8192B 및 서로 다른 SHA를 확인했다. 마지막 페이지와 stress peak는 아직 미검증이다.

마지막 페이지 후속은 UIA의 labelled Spinner RangeValue.Maximum과 표시된 총페이지를 대조한 뒤 해당 값을 직접 선택한다. 실제 readback·본문8KiB 상한·Next 비활성화·PNG를 확인한다. 총페이지를 고정하지 않고 RangeValue 미지원 시 실패를 보존한다. 원90초 worker/소유 HWND/가상 desktop/clipboards 미사용 계약을 유지한다. root는 초안을 검토하고 C# definite-assignment를 수정해 통합했다. Rust/Cargo 변경은 없다.

마지막 페이지 실제 인수 windows-large-report-last-page-20261005-resume도 통과했다. root-review.json에서 source snapshot·파일 SHA·원 DB Failed/단일 Attempt/Artifact 없음·metadata 호출과 source/sentinel 보존·Job0/private desktop 정리를 대조했다. actual RangeValue 및 표시 총236페이지를 확인하고 마지막값236, 본문7982B, first/next와 다른 SHA 및 Next 비활성화 PNG를 직접 확인했다. 대형 보고서 저장·first/next/last 좁은 GUI 인수는 완료이며 stress 비용 및 clipboard/정확2MiB는 별도 미완료다.

최신기본 reports cap privatewriter 추가후largeJSON 재실행은저장전진단header 접근에서FAIL했다. 원source/sentinel·원Job정리는보존됐고원실패증거 windows-report-cap-large-json-current-default-20261005-resume를유지한다. collector BoundsClick는EnsureContent가ScrollIntoView보다앞서있어header Focus/ScrollItem을실행하기전반복wheel로실패했다. root는기존root담당C#에서원ScrollItem/FocusableButton.SetFocus→500msframe→원geometry기록→같은가시성검사→원pointerclick순서로수정한다. 수집기가실제inContent조건을완화하지않으며writer호출전실패를용량정책bug로주장하지않는다. metrics는/tmp읽기감사만담당했다.
