* [Find tax rules by city](#find-tax-rules-by-city)

## Find tax rules by city

#### Finds the tax rules a city keeps, as the editors of the city maintain them.

The rules hold the currency of the city and every tax period, in the order the periods start, each with its tariff bands
in the order of the day, its toll free calendar, its daily maximum, its single charge window, and its exempt vehicles.
Weekdays are ISO 8601 day numbers, 1 being Monday, and months run from 1 to 12.

**GET** `{{congestion-tax-dns}}/tax-rules/{city}`

### Request

**Path and query parameters**:

| Parameter |  Type  | Description                                                                          | Constraints         | Required |
|:----------|:------:|:-------------------------------------------------------------------------------------|:--------------------|:--------:|
| `city`    | String | City whose tax rules are found, matched without regard to case or surrounding spaces. | E.g., `gothenburg`. |   Yes    |

### Response

- `200 OK`

  **Description**: Indicates that the city keeps tax rules.

  **Content-Type**: application/json

  **Body**:
  ```json
  {
      "city": "gothenburg",
      "periods": [
          {
              "starts_on": "2013-01-01",
              "ends_on": "2013-12-31",
              "tariff_bands": [
                  {
                      "amount": 8,
                      "starts_at": "06:00:00",
                      "ends_at": "06:30:00"
                  },
                  {
                      "amount": 13,
                      "starts_at": "06:30:00",
                      "ends_at": "07:00:00"
                  }
              ],
              "exempt_vehicles": [
                  "BUS",
                  "EMERGENCY"
              ],
              "public_holidays": [
                  "2013-01-01",
                  "2013-03-29"
              ],
              "toll_free_months": [
                  7
              ],
              "toll_free_weekdays": [
                  6,
                  7
              ],
              "daily_maximum_amount": 60,
              "single_charge_window_minutes": 60,
              "toll_free_day_before_holiday": true
          }
      ],
      "currency": "SEK"
  }
  ```

- `404 Not Found`

  **Description**: Indicates that the city has no tax rules.

  **Content-Type**: application/json

  **Body**:
  ```json
  {
      "code": "TAX_RULES_NOT_FOUND",
      "message": "No tax rules found for the city."
  }
  ```

- `405 Method Not Allowed`

  **Description**: Indicates that the path exists but does not accept the request method.

  **Content-Type**: application/json

  **Body**:
  ```json
  {
      "code": "METHOD_NOT_ALLOWED",
      "message": "Method Not Allowed."
  }
  ```

- `503 Service Unavailable`

  **Description**: Indicates that the service is not ready to handle requests.

  **Content-Type**: application/json

  **Body**:
  ```json
  {
      "code": "SERVICE_UNAVAILABLE",
      "message": "The service is not ready."
  }
  ```
