# \LolosApi

All URIs are relative to *https://app.lovassy.hu*

Method | HTTP request | Description
------------- | ------------- | -------------
[**api_lolos_get**](LolosApi.md#api_lolos_get) | **GET** /Api/Lolos | Get a list of all lolo coins
[**api_lolos_own_get**](LolosApi.md#api_lolos_own_get) | **GET** /Api/Lolos/Own | Get a list of the user's lolo coins



## api_lolos_get

> Vec<models::ShopIndexLolosResponse> api_lolos_get(filters, sorts, page, page_size)
Get a list of all lolo coins

Requires verified email; Requires one of the following permissions: Shop.IndexLolos; Requires the following features to be enabled: Shop

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**filters** | Option<**String**> |  |  |
**sorts** | Option<**String**> |  |  |
**page** | Option<**i32**> |  |  |
**page_size** | Option<**i32**> |  |  |

### Return type

[**Vec<models::ShopIndexLolosResponse>**](ShopIndexLolosResponse.md)

### Authorization

[Token](../README.md#Token)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## api_lolos_own_get

> models::ShopIndexOwnLolosResponse api_lolos_own_get(filters, sorts, page, page_size)
Get a list of the user's lolo coins

Requires verified email; Requires one of the following permissions: Shop.IndexOwnLolos; Requires the following features to be enabled: Shop

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**filters** | Option<**String**> |  |  |
**sorts** | Option<**String**> |  |  |
**page** | Option<**i32**> |  |  |
**page_size** | Option<**i32**> |  |  |

### Return type

[**models::ShopIndexOwnLolosResponse**](ShopIndexOwnLolosResponse.md)

### Authorization

[Token](../README.md#Token)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

