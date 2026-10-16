# S04: Windows managed reader pure records

- 상태: in-progress
- 분류: safety-format
- 우선순위: P0
- 요구사항: R03, R05, R08, R13
- 담당/수정 범위: diagnostic_cache; 신규 tools/windows_managed_records.rs(게시 GO 대기), 이 문서
- 선행 작업: root module/visibility/schema 통합; Windows native storage/guardian receipts 후속

## 사용자 결과

관리 domain의 reader순서·원anchorsbinding·receipt형식을엄격검증한다. 순수format은원HANDLE/ACL/Jobterminal/delete authority를주지않으며자동회수공개API는없다.

## 계약과 범위

/tmp/dlpad-windows-managed-records.rs만초안작성,Windowscollector freeze중프로젝트Rust게시하지않았다. 원anchors root/marker/lock/journal + fixed8receiptIDs=12개유지. 각IDvolumeu64/fileid16,alias거절. journal JSON16KiB hardcap,slots8singleuse/sequential Empty→Intent→Bound→Resolved. stage VersionProbe/Metadata/Download/DirectProcessing와session/reader nonce·index·guardian/tool FILETIME·originalanchors·recorddigest를domainSHA에포함. receiptMAGIC DLPMRC01/cookieMAGIC분리,256fixedbytes/version/stage/reserved/endingstrictparse. 最大receipt payload2KiB +journal16KiB;filehandle/nativeACL/storage비용은후속도메인,전체stream메모리없음.

## 인수 조건

allEmpty/Intent 및Resolved만으로receipt없는검증false. 외부actualbytes는decode후all_receipt_bytes_match로검증하지만원receiptfileIDs/ACL/writerJobterminal은caller필수. receipt_for는기대값encoding이며native완료증거가아니다. unusedtrailingEmpty는reader未使用でありactualusedreceipts全一致のみ許容、使用Intentは回収不可。누락/중복/wrongnonce/domain/digest/stage/FILETIME/anchors/stateorder/재사용/unknown/oversize/partialreserved변조실패.

## 검증과 handoff

4개 meaningful pure회귀와단독fmtPASS초안READY. Cargo/프로젝트Rust/lib변경없음. root GO후게시/검증. root가process/native storage접근용pub(crate)visibility를조정하며순수pass를Windows native회수완료로확대하지않는다.
